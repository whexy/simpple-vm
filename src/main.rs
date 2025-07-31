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
use payload::{load_dtb, load_uboot};

const FIRMWARE_BASE: u64 = 0x0;
const FIRMWARE_SIZE: u64 = 128 * 1024 * 1024; // 128 MiB for firmware
const MEMORY_BASE: u64 = 0x40000000;
const MEMORY_SIZE: usize = 1024 * 1024 * 1024; // 1 GiB
const UART_BASE: u64 = 0x9000000; // Base address for UART

fn run() -> Result<(), SimppleError> {
    env_logger::init();

    // Setup virtual machine
    let mut virtual_machine: VirtualMachine = VirtualMachine::new(None)?;

    // Setup MMU
    let mut mmu = SharedMemory::default();

    // Main Memory
    mmu.add_segment(
        &mut virtual_machine,
        MEMORY_BASE,
        MEMORY_SIZE,
        MemoryPermission::READ_WRITE_EXECUTE,
    )?;

    // Firmware
    mmu.add_segment(
        &mut virtual_machine,
        FIRMWARE_BASE,
        FIRMWARE_SIZE as usize,
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
        UART_BASE, // Base address for UART
        Box::new(uart_device),
    )?;

    // Setup Debugger
    let debugger = Debugger::new()?;

    // Setup Memory
    let user_payload = load_uboot()?;
    mmu.write_bytes(&mut virtual_machine, FIRMWARE_BASE, user_payload.as_slice())?;

    let dtb_payload = load_dtb()?;
    mmu.write_bytes(&mut virtual_machine, MEMORY_BASE, dtb_payload.as_slice())?;

    // Setup vCPU
    let mut vcpu = virtual_machine.create_vcpu(None)?;

    let mut spsr = SpsrEl3::new();
    spsr.set_condition_flags(false, false, false, false);
    spsr.set_interrupt_masks(true, true, true, true);
    spsr.set_exception_level(1); // EL1
    spsr.set_stack_pointer(false); // Use dedicated stack pointer for EL3

    vcpu.set_register(Register::CPSR, spsr.raw())?;
    vcpu.set_register(Register::PC, FIRMWARE_BASE)?;
    vcpu.set_trap_debug_exceptions(true)?;

    loop {
        let result = vcpu.run()?;
        match result {
            VirtualCpuExitReason::Exception { exception } => {
                // system stopped. show the reason

                let esr_el2 = EsrEl2::from_raw(exception.syndrome);
                match esr_el2.exception_class() {
                    ExceptionClass::DataAbortLowerEl | ExceptionClass::DataAbortSameEl => {
                        let iss = DataAbortISS::from_raw(esr_el2.iss() as u32);

                        match iss.is_write() {
                            true => {
                                mmio_manager
                                    .handle_write(
                                        exception.physical_address,
                                        iss.access_size().into(),
                                        get_register_value(&mut vcpu, iss.access_register())?,
                                    )
                                    .inspect_err(|_| {
                                        let _ = debugger.print_debug_info(
                                            &virtual_machine,
                                            &mut vcpu,
                                            &mmu,
                                        );
                                    })?;
                            }
                            false => {
                                let value = mmio_manager
                                    .handle_read(
                                        exception.physical_address,
                                        iss.access_size().into(),
                                    )
                                    .inspect_err(|_| {
                                        let _ = debugger.print_debug_info(
                                            &virtual_machine,
                                            &mut vcpu,
                                            &mmu,
                                        );
                                    })?;
                                set_register_value(&mut vcpu, iss.access_register(), value)?;
                            }
                        }
                    }
                    ExceptionClass::HvcAArch64 => {
                        debugger.print_debug_info(&virtual_machine, &mut vcpu, &mmu)?;
                        log::info!("HVC instruction executed successfully.");
                        break;
                    }
                    exception_class => {
                        debugger.print_debug_info(&virtual_machine, &mut vcpu, &mmu)?;
                        log::error!("unexpected exception: {exception_class:?}");
                        break;
                    }
                };
            }
            reason => {
                debugger.print_debug_info(&virtual_machine, &mut vcpu, &mmu)?;
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
