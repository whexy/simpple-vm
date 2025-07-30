use ahv::*;
use anyhow::Result;
use simpple_vm::debugger::Debugger;
use simpple_vm::esr_el2::ExceptionClass;
use simpple_vm::mems::SharedMemory;
use simpple_vm::regs::SpsrEl3;
use simpple_vm::{DataAbortISS, EsrEl2, MmioManager, SimppleError};

mod payload;
use payload::gen_payload;

const PAYLOAD_ADDR: u64 = 0x20000;

fn run() -> Result<(), SimppleError> {
    env_logger::init();

    // Setup virtual machine
    let mut virtual_machine: VirtualMachine = VirtualMachine::new(None)?;

    // Setup MMU
    let mut mmu = SharedMemory::default();
    mmu.add_segment(
        &mut virtual_machine,
        PAYLOAD_ADDR,
        64 * 1024, // 64 KiB for the payload
        MemoryPermission::EXECUTE,
    )?;

    // Setup MMIO Manager
    let mut mmio_manager = MmioManager::default();

    // Setup Debugger
    let debugger = Debugger::new()?;

    // Setup code segment
    let user_payload = gen_payload()?;
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

                        mmio_manager.handle_access(
                            exception.physical_address,
                            iss.access_size().into(),
                            iss.is_write(),
                            if iss.is_write() {
                                Some(vcpu.get_register(iss.access_register())?)
                            } else {
                                None
                            },
                        )?;

                        if !iss.is_write() {
                            vcpu.set_register(iss.access_register(), 0x42)?;
                        }
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
