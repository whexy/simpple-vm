use crate::MmioError;

pub trait Register {
    fn read(&self) -> u64;
    fn write(&mut self, value: u64, size: usize) -> Result<(), MmioError>;
    fn reset(&mut self);
}

#[derive(Debug, Clone)]
pub struct RwRegister {
    value: u64,
    mask: u64, // writable bits mask
}

impl RwRegister {
    pub fn new(initial_value: u64, writable_mask: u64) -> Self {
        Self {
            value: initial_value,
            mask: writable_mask,
        }
    }
}

impl Register for RwRegister {
    fn read(&self) -> u64 {
        self.value
    }

    fn write(&mut self, value: u64, _size: usize) -> Result<(), MmioError> {
        self.value = (self.value & !self.mask) | (value & self.mask);
        Ok(())
    }

    fn reset(&mut self) {
        self.value = 0;
    }
}

#[derive(Debug, Clone)]
pub struct RoRegister {
    value: u64,
}

impl RoRegister {
    pub fn new(initial_value: u64) -> Self {
        Self {
            value: initial_value,
        }
    }

    pub fn set_value(&mut self, value: u64) {
        self.value = value;
    }
}

impl Register for RoRegister {
    fn read(&self) -> u64 {
        self.value
    }

    fn write(&mut self, _value: u64, _size: usize) -> Result<(), MmioError> {
        // just ignore writes
        Ok(())
    }

    fn reset(&mut self) {
        self.value = 0
    }
}
