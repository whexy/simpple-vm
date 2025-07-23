use ahv::*;
use anyhow::Result;
use keystone_engine::{Arch, Keystone, Mode};
use simpple_vm::regs::SpsrEl3;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Hypervisor error: {0:?}")]
    Hypervisor(HypervisorError),
    #[error("Keystone error: {0}")]
    Keystone(#[from] keystone_engine::KeystoneError),
    #[error("Capstone error: {0}")]
    Capstone(#[from] capstone::Error),
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<HypervisorError> for VmError {
    fn from(err: HypervisorError) -> Self {
        VmError::Hypervisor(err)
    }
}

fn gen_payload() -> Result<Vec<u8>> {
    let engine = Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN)?;

    let asm = r#"
        .global _start
        _start:
            mov x0, #42
            add x0, x0, #3
            
            // Try to access unmapped memory at 0x09000000 (UART region)
            mov x1, #0x09000000
            ldr w2, [x1]        // This should cause a data abort
            
            hvc #0
            ret
    "#;

    let result = engine.asm(asm.to_string(), 0)?;

    let instruction_view: Vec<_> = result
        .bytes
        .chunks_exact(4)
        .enumerate()
        .map(|(i, chunk)| {
            let instr = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            format!("   [{i}]: 0x{instr:08x}")
        })
        .collect();
    log::info!(
        "Payload generated ({} instructions):\n{}",
        result.bytes.len() / 4,
        instruction_view.join("\n")
    );

    Ok(result.bytes)
}

const PAYLOAD_ADDR: hv_ipa_t = 0x20000;

fn main() -> Result<(), VmError> {
    env_logger::init();

    let user_payload = gen_payload()?;

    let mut virtual_machine: VirtualMachine = VirtualMachine::new(None)?;
    let payload_allocation = virtual_machine.allocate_from(&user_payload)?;
    virtual_machine.map(
        payload_allocation,
        PAYLOAD_ADDR,
        MemoryPermission::READ_WRITE_EXECUTE,
    )?;

    {
        let mut vcpu = virtual_machine.create_vcpu(None)?;

        let mut spsr = SpsrEl3::new();
        spsr.set_condition_flags(false, false, false, false);
        spsr.set_interrupt_masks(true, true, true, true);
        spsr.set_exception_level(1); // EL1
        spsr.set_stack_pointer(false); // Use dedicated stack pointer for EL3

        vcpu.set_register(Register::CPSR, spsr.raw())?;
        vcpu.set_register(Register::PC, PAYLOAD_ADDR)?;
        vcpu.set_trap_debug_exceptions(true)?;

        let result = vcpu.run()?;
        match result {
            VirtualCpuExitReason::Exception { exception } => {
                match (exception.syndrome >> 26) & 0x3f {
                    0x16 => {
                        log::info!("HVC instruction executed successfully.");
                    }
                    0x24 => {
                        log::info!("Data Abort exception occurred.");
                        log::info!("Address: {:#x}", exception.virtual_address);
                    }
                    _ => {
                        log::warn!("unexpected exception: {:#x}", exception.syndrome);
                        println!("exception: {exception:?}")
                    }
                };
            }
            reason => {
                log::error!("Unexpected exit reason: {reason:?}");
            }
        };

        println!("x0: {:#x}", vcpu.get_register(Register::X0)?);
    }

    Ok(())
}
