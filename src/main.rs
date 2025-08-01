use ahvf::*;
use anyhow::Result;
use simpple_vm::debugger::Debugger;
use simpple_vm::devices::gpio::Pl061Gpio;
use simpple_vm::devices::timer::get_cntpct_el0;
use simpple_vm::devices::uart::Pl011Device;
use simpple_vm::mems::SharedMemory;
use simpple_vm::regs::iss::{DataAbortISS, SysRegAbortISS};
use simpple_vm::regs::utils::{get_register_value, set_register_value};
use simpple_vm::regs::{EmulatedSystemRegister, EsrEl2, ExceptionClass, SpsrEl3};
use simpple_vm::{MmioManager, SimppleError};

mod payload;
use payload::{load_dtb, load_uboot};

const FIRMWARE_BASE: u64 = 0x0;
const FIRMWARE_SIZE: usize = 128 * 1024 * 1024; // 128 MiB for firmware
const MEMORY_BASE: u64 = 0x40000000;
const MEMORY_SIZE: usize = 1024 * 1024 * 1024; // 1GiB of memory
const UART_BASE: u64 = 0x9000000; // Base address for UART
const GPIO_BASE: u64 = 0x3fffe000;

fn run() -> Result<(), SimppleError> {
    let mut virtual_machine = VirtualMachine::new(None)?;

    // Setup MMU
    let mut mmu = SharedMemory::default();

    // Main Memory
    mmu.add_segment(
        &mut virtual_machine,
        FIRMWARE_BASE,
        FIRMWARE_SIZE,
        MemoryPermission::READ_WRITE_EXECUTE,
    )?;

    mmu.add_segment(
        &mut virtual_machine,
        MEMORY_BASE,
        MEMORY_SIZE,
        MemoryPermission::READ_WRITE_EXECUTE,
    )?;

    // Setup devices
    let mut mmio_manager = MmioManager::default();
    let uart_device = Pl011Device::stdout();

    mmio_manager.register_device(
        UART_BASE, // Base address for UART
        Box::new(uart_device),
    )?;

    let gpio_device = Pl061Gpio::default();
    mmio_manager.register_device(
        GPIO_BASE, // Base address for GPIO
        Box::new(gpio_device),
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

    vcpu.set_vtimer_mask(false)?;

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
                                let mmio_result = mmio_manager.handle_write(
                                    exception.physical_address,
                                    iss.access_size().into(),
                                    get_register_value(&mut vcpu, iss.access_register())?,
                                );
                                match mmio_result {
                                    Ok(_) => {}
                                    Err(e) => {
                                        log::error!(
                                            "{e}: invalid read from {:#0x}",
                                            exception.physical_address
                                        );
                                        // let _ = debugger.print_debug_info(
                                        //     &virtual_machine,
                                        //     &mut vcpu,
                                        //     &mmu,
                                        // );
                                    }
                                }
                            }
                            false => {
                                let mmio_result = mmio_manager.handle_read(
                                    exception.physical_address,
                                    iss.access_size().into(),
                                );
                                match mmio_result {
                                    Ok(value) => {
                                        set_register_value(
                                            &mut vcpu,
                                            iss.access_register(),
                                            value,
                                        )?;
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "{e}: invalid write to {:#0x}",
                                            exception.physical_address
                                        );
                                        let _ = debugger.print_debug_info(
                                            &virtual_machine,
                                            &mut vcpu,
                                            &mmu,
                                        );
                                    }
                                };
                            }
                        }
                    }
                    ExceptionClass::HvcAArch64 => {
                        debugger.print_debug_info(&virtual_machine, &mut vcpu, &mmu)?;
                        log::info!("HVC instruction executed successfully.");
                        break;
                    }
                    ExceptionClass::TrappedSysregAArch64 => {
                        let iss = SysRegAbortISS::from_raw(esr_el2.iss() as u32);

                        let system_register = iss.system_register();
                        let gp_register = iss.access_register();
                        log::info!(
                            "Accessing system register: {system_register:?} using {gp_register:?}"
                        );

                        match system_register {
                            EmulatedSystemRegister::CntpCtEl0 => {
                                let value = get_cntpct_el0();
                                set_register_value(&mut vcpu, gp_register, value)?;
                                log::info!("Successfully emulating accessed CntpCtEl0: {value:#x}");
                            }
                        }
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
                break;
            }
        };

        let pc_addr = vcpu.get_register(Register::PC)?;
        vcpu.set_register(Register::PC, pc_addr + 4)?; // PC += 4
    }

    Ok(())
}

fn main() {
    env_logger::init();
    match run() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{e}");
        }
    }
}
