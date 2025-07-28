use ahv::*;
use anyhow::Result;
use capstone::prelude::*;
use keystone_engine::{Arch, Keystone, Mode};
use simpple_vm::esr_el2::ExceptionClass;
use simpple_vm::mems::SharedMemory;
use simpple_vm::regs::SpsrEl3;
use simpple_vm::{DataAbortISS, EsrEl2, SimppleError};

fn gen_payload() -> Result<Vec<u8>> {
    let engine = Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN)?;

    let asm = r#"
        .global _start
        _start:
            mov x0, #42
            add x0, x0, #3
            
            // Try to access unmapped memory at 0x09000000 (UART region)
            mov x1, #0x09000000
            ldr x2, [x1]        // This should cause a data abort
            mov x3, x2
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

fn decode_payload(payload: &[u8]) -> Result<()> {
    let cs = Capstone::new()
        .arm64()
        .mode(arch::arm64::ArchMode::Arm)
        .detail(true)
        .build()?;

    let instructions = cs.disasm_all(payload, 0x0)?;
    for insn in instructions.iter() {
        println!(
            "{:08x}:\t{}\t{}",
            insn.address(),
            insn.mnemonic().unwrap_or(""),
            insn.op_str().unwrap_or("")
        );

        let detail = cs.insn_detail(insn)?;
        println!("Detail: {detail:?}");
        // println!("  Write Registers: {}", reg_names(&cs, detail.regs_write()));
        // println!("  Read Registers: {}", reg_names(&cs, detail.regs_read()));
    }
    Ok(())
}

const PAYLOAD_ADDR: hv_ipa_t = 0x20000;

fn main() -> Result<(), SimppleError> {
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
                let esr_el2 = EsrEl2::from_raw(exception.syndrome);
                match esr_el2.exception_class() {
                    ExceptionClass::HvcAArch64 => {
                        log::info!("HVC instruction executed successfully.");
                        // print out the debug info
                        println!("x0: {:#x}", vcpu.get_register(Register::X0)?);
                        println!("x1: {:#x}", vcpu.get_register(Register::X1)?);
                        println!("x2: {:#x}", vcpu.get_register(Register::X2)?);
                        println!("x3: {:#x}", vcpu.get_register(Register::X3)?);
                        println!("x4: {:#x}", vcpu.get_register(Register::X4)?);
                        println!("x5: {:#x}", vcpu.get_register(Register::X5)?);
                        println!("x6: {:#x}", vcpu.get_register(Register::X6)?);
                        println!("x7: {:#x}", vcpu.get_register(Register::X7)?);
                        println!("x8: {:#x}", vcpu.get_register(Register::X8)?);
                        println!("x9: {:#x}", vcpu.get_register(Register::X9)?);
                        println!("x10: {:#x}", vcpu.get_register(Register::X10)?);
                        println!("x11: {:#x}", vcpu.get_register(Register::X11)?);
                        println!("x12: {:#x}", vcpu.get_register(Register::X12)?);
                        println!("x13: {:#x}", vcpu.get_register(Register::X13)?);
                        println!("x14: {:#x}", vcpu.get_register(Register::X14)?);
                        println!("x15: {:#x}", vcpu.get_register(Register::X15)?);
                        println!("x16: {:#x}", vcpu.get_register(Register::X16)?);
                        println!("x17: {:#x}", vcpu.get_register(Register::X17)?);
                        println!("x18: {:#x}", vcpu.get_register(Register::X18)?);
                        println!("x19: {:#x}", vcpu.get_register(Register::X19)?);
                        println!("x20: {:#x}", vcpu.get_register(Register::X20)?);
                        println!("x21: {:#x}", vcpu.get_register(Register::X21)?);
                        println!("x22: {:#x}", vcpu.get_register(Register::X22)?);
                        println!("x23: {:#x}", vcpu.get_register(Register::X23)?);
                        println!("x24: {:#x}", vcpu.get_register(Register::X24)?);
                        println!("x25: {:#x}", vcpu.get_register(Register::X25)?);
                        println!("x26: {:#x}", vcpu.get_register(Register::X26)?);
                        println!("x27: {:#x}", vcpu.get_register(Register::X27)?);
                        println!("x28: {:#x}", vcpu.get_register(Register::X28)?);
                        println!("x29: {:#x}", vcpu.get_register(Register::X29)?);
                        println!("x30: {:#x}", vcpu.get_register(Register::X30)?);
                        println!("x30: {:#x}", vcpu.get_register(Register::X30)?);
                        break;
                    }
                    ExceptionClass::DataAbortLowerEl | ExceptionClass::DataAbortSameEl => {
                        log::info!("Data Abort exception occurred.");
                        log::info!("Address: {:#x}", exception.virtual_address);

                        let iss = DataAbortISS::from_raw(esr_el2.iss() as u32);
                        if iss.is_write() {
                            log::info!("Write value to register: {:?}", iss.access_register());
                            vcpu.set_register(iss.access_register(), 0x42)?;
                        }
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
        println!("PC: {:#x}", vcpu.get_register(Register::PC)?);

        let pc_addr = vcpu.get_register(Register::PC)?;

        let instr = mmu.read_bytes(&virtual_machine, pc_addr, 4)?;
        decode_payload(instr.as_slice())?;

        vcpu.set_register(Register::PC, pc_addr + 4)?; // PC += 4
    }

    Ok(())
}
