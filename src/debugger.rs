use crate::regs::SpsrEl3;
use crate::{SharedMemory, SimppleError};
use ahvf::*;
use anyhow::Result;
use capstone::prelude::*;
use colored::{ColoredString, Colorize};

pub struct Debugger {
    cs: capstone::Capstone,
}

impl Debugger {
    pub fn new() -> Result<Self> {
        let cs = Capstone::new()
            .arm64()
            .mode(arch::arm64::ArchMode::Arm)
            .detail(true)
            .build()?;
        Ok(Debugger { cs })
    }

    pub fn decode(&self, payload: &[u8], address: u64) -> Result<()> {
        let instructions = self.cs.disasm_all(payload, address)?;
        for insn in instructions.iter() {
            let insn_bytes = insn.bytes();
            let insn_repr =
                u32::from_le_bytes([insn_bytes[0], insn_bytes[1], insn_bytes[2], insn_bytes[3]]);
            println!(
                "{:08x}:\t{:#0x}\t{}\t{}",
                insn.address(),
                insn_repr,
                insn.mnemonic().unwrap_or(""),
                insn.op_str().unwrap_or("")
            );
        }
        Ok(())
    }

    pub fn print_debug_info(
        &self,
        vm: &VirtualMachine,
        vcpu: &mut VirtualCpu,
        mmu: &SharedMemory,
    ) -> Result<(), SimppleError> {
        println!(
            "{}",
            "==================== Debugger ===================="
                .bright_cyan()
                .bold()
        );

        let cpsr = vcpu.get_register(Register::CPSR)?;
        let spsr = SpsrEl3::from_raw(cpsr);

        match spsr.exception_level() {
            0 => println!("Current Exception Level: EL0"),
            1 => println!("{}", "Current Exception Level: EL1".bright_yellow()),
            2 => println!("{}", "Current Exception Level: EL2".bright_blue()),
            3 => println!("{}", "Current Exception Level: EL3".bright_red()),
            _ => unreachable!(),
        }

        let pc_addr = vcpu.get_register(Register::PC)?;

        // Display instructions: 4 before, current, 4 after
        self.print_instructions_around_pc(vm, mmu, pc_addr)?;

        println!(
            "{}",
            "--------------------------------------------------".bright_cyan()
        );

        // Print registers in grid format
        self.print_gp_registers_grid(vcpu)?;

        Ok(())
    }

    fn print_instructions_around_pc(
        &self,
        vm: &VirtualMachine,
        mmu: &SharedMemory,
        pc_addr: u64,
    ) -> Result<(), SimppleError> {
        const INSTRUCTION_SIZE: u64 = 4; // ARM64 instructions are 4 bytes
        const CONTEXT_INSTRUCTIONS: u64 = 2;

        let start_addr = pc_addr.saturating_sub(CONTEXT_INSTRUCTIONS * INSTRUCTION_SIZE);

        let total_instructions = CONTEXT_INSTRUCTIONS * 2 + 1;
        let total_bytes = total_instructions * INSTRUCTION_SIZE;

        match mmu.read_bytes(vm, start_addr, total_bytes as usize) {
            Ok(instrs) => {
                if let Ok(instructions) = self.cs.disasm_all(&instrs, start_addr) {
                    for insn in instructions.iter() {
                        if insn.address() == pc_addr {
                            // Highlight current instruction
                            println!(
                                "{} {}",
                                "►".bright_yellow().bold(),
                                format_instruction(insn, true)
                            );
                        } else {
                            println!("  {}", format_instruction(insn, false));
                        }
                    }
                } else {
                    // Fallback: just show current instruction if disassembly fails
                    if let Ok(current_instrs) = mmu.read_bytes(vm, pc_addr, 4) {
                        if let Ok(instructions) = self.cs.disasm_all(&current_instrs, pc_addr) {
                            for insn in instructions.iter() {
                                println!(
                                    "{} {}",
                                    "►".bright_yellow().bold(),
                                    format_instruction(insn, true)
                                );
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Fallback if we can't read the full context
                if let Ok(current_instrs) = mmu.read_bytes(vm, pc_addr, 4) {
                    if let Ok(instructions) = self.cs.disasm_all(&current_instrs, pc_addr) {
                        for insn in instructions.iter() {
                            println!(
                                "{} {}",
                                "►".bright_yellow().bold(),
                                format_instruction(insn, true)
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn print_gp_registers_grid(&self, vcpu: &mut VirtualCpu) -> Result<(), SimppleError> {
        println!("{}", "Registers:".bright_magenta().bold());

        // Print registers in a 4-column grid for better readability
        const COLUMNS: usize = 4;
        let mut register_values = Vec::new();

        // Collect all register values
        for reg in GP_REGISTERS.iter() {
            let value = vcpu.get_register(*reg)?;
            register_values.push((*reg, value));
        }

        // Print in grid format
        for chunk in register_values.chunks(COLUMNS) {
            let mut line = String::new();
            for (reg, value) in chunk {
                let reg_str = format!("{reg:?}");
                let colored_reg = format_register_name(&reg_str);
                let colored_value = format_register_value(*value);
                let column_text = &format!("{colored_reg}:{colored_value}");
                line.push_str(&format!("{column_text:>42}"));
            }
            println!("  {line}");
        }

        Ok(())
    }
}

fn format_instruction(insn: &capstone::Insn, is_current: bool) -> ColoredString {
    let insn_bytes = insn.bytes();
    let insn_repr =
        u32::from_le_bytes([insn_bytes[0], insn_bytes[1], insn_bytes[2], insn_bytes[3]]);

    let instruction_text = format!(
        "{:08x}:\t{:#0x}\t{}\t{}",
        insn.address(),
        insn_repr,
        insn.mnemonic().unwrap_or(""),
        insn.op_str().unwrap_or("")
    );

    if is_current {
        instruction_text.bright_yellow().bold()
    } else {
        instruction_text.normal()
    }
}

fn format_register_name(reg_name: &str) -> ColoredString {
    match reg_name {
        name if name.starts_with("X0") || name.starts_with("X1") => name.bright_green(),
        name if name.starts_with("X2") || name.starts_with("X3") => name.bright_blue(),
        name if name.contains("29") || name.contains("30") => name.bright_yellow(), // FP, LR
        _ => reg_name.bright_magenta(),
    }
}

fn format_register_value(value: u64) -> ColoredString {
    // Color-code values based on their likely significance
    match value {
        0 => "0x0000000000000000".bright_black(),
        v if v < 0x1000 => format!("{v:#018x}").bright_red(), // Likely small integers
        v if v >= 0x7000000000000000 => format!("{v:#018x}").bright_cyan(), // Likely addresses in high memory
        v => format!("{v:#018x}").white(),
    }
}

const GP_REGISTERS: [Register; 32] = [
    Register::X0,
    Register::X1,
    Register::X2,
    Register::X3,
    Register::X4,
    Register::X5,
    Register::X6,
    Register::X7,
    Register::X8,
    Register::X9,
    Register::X10,
    Register::X11,
    Register::X12,
    Register::X13,
    Register::X14,
    Register::X15,
    Register::X16,
    Register::X17,
    Register::X18,
    Register::X19,
    Register::X20,
    Register::X21,
    Register::X22,
    Register::X23,
    Register::X24,
    Register::X25,
    Register::X26,
    Register::X27,
    Register::X28,
    Register::X29,
    Register::X30,
    Register::PC,
];
