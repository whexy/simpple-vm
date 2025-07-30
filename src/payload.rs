use anyhow::Result;
use keystone_engine::{Arch, Keystone, Mode};

use std::fs;

pub fn gen_payload() -> Result<Vec<u8>> {
    let engine = Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN)?;

    let asm = include_str!("../tests/integration/uart.S");

    let result = engine.asm(asm.to_string(), 0)?;
    Ok(result.bytes)
}

pub fn load_uboot() -> Result<Vec<u8>> {
    let uboot_binary =
        fs::read("tests/integration/u-boot.bin").expect("Failed to read uboot binary");
    log::info!("Loaded uboot binary of size: {}", uboot_binary.len());

    Ok(uboot_binary)
}
