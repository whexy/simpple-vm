use ahvf::*;

#[derive(Debug)]
pub enum VRegister {
    Register(Register),
    ZeroRegister,
}

pub fn get_register_value(vcpu: &mut VirtualCpu, vreg: VRegister) -> Result<u64> {
    match vreg {
        VRegister::Register(reg) => Ok(vcpu.get_register(reg)?),
        VRegister::ZeroRegister => Ok(0),
    }
}

pub fn set_register_value(vcpu: &mut VirtualCpu, vreg: VRegister, value: u64) -> Result<()> {
    match vreg {
        VRegister::Register(reg) => vcpu.set_register(reg, value),
        VRegister::ZeroRegister => Ok(()), // Zero register is read-only and always zero
    }
}

#[derive(Debug)]
pub enum EmulatedSystemRegister {
    CntpCtEl0,
}
