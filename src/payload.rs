use anyhow::Result;
use keystone_engine::{Arch, Keystone, Mode};

pub fn gen_payload() -> Result<Vec<u8>> {
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
    Ok(result.bytes)
}
