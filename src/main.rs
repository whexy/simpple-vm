use ahv::*;
use anyhow::Result;
use simpple_vm::debugger::Debugger;
use simpple_vm::esr_el2::ExceptionClass;
use simpple_vm::mems::SharedMemory;
use simpple_vm::regs::SpsrEl3;
use simpple_vm::uart::Pl011Device;
use simpple_vm::utils::{get_register_value, set_register_value};
use simpple_vm::{DataAbortISS, EsrEl2, MmioManager, SimppleError};
use std::io::Write;
use std::sync::{Arc, Mutex};

mod payload;
use payload::load_uboot;

const PAYLOAD_ADDR: u64 = 0x40000000;
const MEMORY_SIZE: usize = 256 * 1024 * 1024; // 256 MiB

fn run() -> Result<(), SimppleError> {
    env_logger::init();

    // Setup virtual machine
    let mut virtual_machine: VirtualMachine = VirtualMachine::new(None)?;

    // Setup MMU
    let mut mmu = SharedMemory::default();
    mmu.add_segment(
        &mut virtual_machine,
        PAYLOAD_ADDR,
        MEMORY_SIZE,
        MemoryPermission::READ_WRITE_EXECUTE,
    )?;

    // Setup devices
    let mut mmio_manager = MmioManager::default();
    let mut uart_device = Pl011Device::default();
    let output_buffer = Arc::new(Mutex::new(Vec::new()));
    let output_buffer_clone = output_buffer.clone();
    uart_device.set_output_handler(move |data| {
        let mut buffer = output_buffer_clone.lock().unwrap();
        buffer.push(data);

        // Print u-boot output in real time
        if data == b'\n' || buffer.len() > 80 {
            let line = String::from_utf8_lossy(&buffer);
            print!("U-boot: {line}");
            std::io::stdout().flush().unwrap();
            buffer.clear();
        }
    });

    mmio_manager.register_device(
        0x09000000, // Base address for UART
        Box::new(uart_device),
    )?;

    // Setup Debugger
    let debugger = Debugger::new()?;

    // Setup code segment
    let user_payload = load_uboot()?;
    mmu.write_bytes(&mut virtual_machine, PAYLOAD_ADDR, user_payload.as_slice())?;

    // Setup vCPU
    let mut vcpu = virtual_machine.create_vcpu(None)?;

    let mut spsr = SpsrEl3::new();
    spsr.set_condition_flags(false, false, false, false);
    spsr.set_interrupt_masks(true, true, true, true);
    spsr.set_exception_level(1); // EL1
    spsr.set_stack_pointer(false); // Use dedicated stack pointer for EL3

    vcpu.set_register(Register::CPSR, spsr.raw())?;
    vcpu.set_register(Register::PC, PAYLOAD_ADDR)?;
    vcpu.set_trap_debug_exceptions(true)?;

    loop {
        let result = vcpu.run()?;
        match result {
            VirtualCpuExitReason::Exception { exception } => {
                // system stopped. show the reason
                debugger.print_debug_info(&virtual_machine, &mut vcpu, &mmu)?;

                let esr_el2 = EsrEl2::from_raw(exception.syndrome);
                match esr_el2.exception_class() {
                    ExceptionClass::DataAbortLowerEl | ExceptionClass::DataAbortSameEl => {
                        let iss = DataAbortISS::from_raw(esr_el2.iss() as u32);

                        let result = mmio_manager.handle_access(
                            exception.physical_address,
                            iss.access_size().into(),
                            iss.is_write(),
                            if iss.is_write() {
                                Some(get_register_value(&mut vcpu, iss.access_register())?)
                            } else {
                                None
                            },
                        )?;

                        if !iss.is_write() {
                            set_register_value(&mut vcpu, iss.access_register(), result.unwrap())?;
                        }

                        log::info!("Output Buffr: {:?}", output_buffer.lock().unwrap());
                    }
                    ExceptionClass::HvcAArch64 => {
                        log::info!("HVC instruction executed successfully.");
                        break;
                    }
                    exception_class => {
                        log::error!("unexpected exception: {exception_class:?}");
                        break;
                    }
                };
            }
            reason => {
                log::error!("Unexpected exit reason: {reason:#?}");
            }
        };

        let pc_addr = vcpu.get_register(Register::PC)?;
        vcpu.set_register(Register::PC, pc_addr + 4)?; // PC += 4
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{e}");
        }
    }
}
