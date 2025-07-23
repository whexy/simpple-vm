use ahv::*;
use keystone_engine::{Arch, Keystone, Mode};

const CODE: hv_ipa_t = 0x20000;
const DATA: hv_ipa_t = 0x40000;

fn gen_payload() -> Vec<u8> {
    let engine = Keystone::new(Arch::ARM64, Mode::LITTLE_ENDIAN).unwrap();
    let asm = r#"
        .global _start
        _start:
            mov x0, #42
            add x0, x0, #3
    "#;
    let result = engine.asm(asm.to_string(), 0).unwrap();
    result.bytes
}

fn main() -> Result<()> {
    let payload = gen_payload();

    let mut vm = VirtualMachine::new(None)?;
    let payload_handle = vm.allocate_from(&payload)?;
    vm.map(payload_handle, CODE, MemoryPermission::EXECUTE)?;

    let data = vec![0u8; 1024]; // 1KB
    let data_handle = vm.allocate_from(&data)?;
    vm.map(data_handle, DATA, MemoryPermission::READ_WRITE)?;

    let mut vcpu = vm.create_vcpu(None)?;
    vcpu.set_register(Register::CPSR, 0x3c4)?; // EL1t
    vcpu.set_register(Register::PC, CODE)?;

    let _ = vcpu.run()?;
    println!("x0 is {}", vcpu.get_register(Register::X0)?);
    Ok(())
}
