use crate::err::MmioError;

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
        self.value = 0; // Reset to zero
    }
}

/// Write-only register with side effects
pub struct WoRegister<F>
where
    F: FnMut(u64),
{
    write_handler: F,
}

impl<F> WoRegister<F>
where
    F: FnMut(u64),
{
    pub fn new(write_handler: F) -> Self {
        Self { write_handler }
    }
}

impl<F> Register for WoRegister<F>
where
    F: FnMut(u64),
{
    fn read(&self) -> u64 {
        0 // Write-only registers return 0 on read
    }

    fn write(&mut self, value: u64, _size: usize) -> Result<(), MmioError> {
        (self.write_handler)(value);
        Ok(())
    }

    fn reset(&mut self) {
        // No state to reset for write-only registers
    }
}
