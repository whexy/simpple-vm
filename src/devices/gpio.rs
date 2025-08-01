//! ARM PrimeCell PL061 General Purpose I/O (GPIO) device emulation.
//!
//! This module provides a minimal implementation of a PL061 GPIO controller,
//! sufficient to satisfy the probe sequence from a guest OS like U-Boot when
//! running on a QEMU `virt` machine profile. It emulates the core data and
//! direction registers for 8 GPIO pins and correctly reports its peripheral ID.
//! Interrupt functionality is stubbed out.

use crate::devices::MmioDevice;
use crate::err::MmioError;

// --- ARM PL061 Register Offsets ---
// These are byte offsets from the base address.
const GPIODATA: u64 = 0x000; // Data Register (up to 0x3FC for specific bit access)
const GPIODIR: u64 = 0x400; // Direction Register
const GPIOIS: u64 = 0x404; // Interrupt Sense Register
const GPIOIBE: u64 = 0x408; // Interrupt Both Edges Register
const GPIOIEV: u64 = 0x40C; // Interrupt Event Register
const GPIOIE: u64 = 0x410; // Interrupt Mask Register
const GPIORIS: u64 = 0x414; // Raw Interrupt Status Register
const GPIOMIS: u64 = 0x418; // Masked Interrupt Status Register
const GPIOIC: u64 = 0x41C; // Interrupt Clear Register
const GPIOAFSEL: u64 = 0x420; // Alternate Function Select Register

const GPIO_PERIPH_ID_BASE: u64 = 0xFE0; // Start of Peripheral ID registers
const GPIO_PCELL_ID_BASE: u64 = 0xFF0; // Start of PrimeCell ID registers

/// Standard ARM PL061 Peripheral & PrimeCell IDs.
/// The Peripheral ID is bytes 0-7, and the PrimeCell ID is bytes 8-11.
const PL061_IDS: [u8; 12] = [
    // Peripheral ID (0xFE0 - 0xFEC)
    0x61, 0x10, 0x04, 0x00, // PrimeCell ID (part of Peripheral ID space in PL061)
    0x0d, 0xf0, 0x05, 0xb1, // PrimeCell ID (0xFF0 - 0xFFC)
    0x0d, 0xf0, 0x05, 0xb1,
];

/// ARM PL061 GPIO device state.
///
/// This struct emulates an 8-bit GPIO controller.
pub struct Pl061Gpio {
    /// State of the 8 GPIO pins. A '1' means high, '0' means low.
    data: u8,
    /// Direction for each of the 8 pins. A '1' means output, '0' means input.
    direction: u8,
    /// Interrupt enable state.
    interrupt_enable: u8,
    /// Alternate function select state.
    afsel: u8,
}

impl Pl061Gpio {
    /// Creates a new PL061 GPIO device in its reset state.
    pub fn new() -> Self {
        Self {
            // All pins are low and configured as inputs at reset.
            data: 0,
            direction: 0,
            interrupt_enable: 0,
            afsel: 0,
        }
    }

    /// Reads a byte from the combined ID array.
    fn get_id_byte(&self, offset: u64) -> u64 {
        // The ID registers are contiguous from 0xFE0 to 0xFFC.
        let index = (offset - GPIO_PERIPH_ID_BASE) as usize;
        if index < PL061_IDS.len() {
            u64::from(PL061_IDS[index])
        } else {
            0
        }
    }
}

impl Default for Pl061Gpio {
    fn default() -> Self {
        Self::new()
    }
}

impl MmioDevice for Pl061Gpio {
    /// Handles a read from a GPIO register.
    fn read(&mut self, offset: u64, size: usize) -> Result<u64, MmioError> {
        // The guest may use 8-byte accesses (e.g., LDP instruction), so we must
        // handle them gracefully. We allow power-of-two sizes up to 8 bytes.
        // Since all PL061 registers are 32-bits or smaller, for any valid access
        // size, we return the register value zero-extended to 64 bits. The
        // hypervisor is expected to handle truncation if necessary.
        if size > 8 || size == 0 || (size & (size - 1)) != 0 {
            return Err(MmioError::InvalidSize { size });
        }

        let value = match offset {
            // Data register: returns the current state of all 8 pins.
            // The spec allows for masked access from 0x000 to 0x3FC, but we
            // implement the simplified 0x000 access that returns the whole byte.
            0x000..=0x3FC => u64::from(self.data),

            // Direction register.
            GPIODIR => u64::from(self.direction),

            // Interrupt and AFSEL registers.
            GPIOIE => u64::from(self.interrupt_enable),
            GPIOAFSEL => u64::from(self.afsel),

            // Stubbed read-only interrupt status registers. Always return 0 (no interrupts).
            GPIOIS | GPIOIBE | GPIOIEV | GPIORIS | GPIOMIS => 0,

            // Peripheral and PrimeCell ID registers. This is the crucial part
            // for satisfying the guest's probe.
            GPIO_PERIPH_ID_BASE..=0xFFC => self.get_id_byte(offset),

            _ => {
                // Per the spec, reads to undefined registers should return 0.
                0
            }
        };

        Ok(value)
    }

    /// Handles a write to a GPIO register.
    fn write(&mut self, offset: u64, size: usize, value: u64) -> Result<(), MmioError> {
        // The guest may use 8-byte accesses (e.g., STP instruction).
        // We allow power-of-two sizes up to 8 bytes.
        if size > 8 || size == 0 || (size & (size - 1)) != 0 {
            return Err(MmioError::InvalidSize { size });
        }

        let byte_value = value as u8;

        match offset {
            // Data register. A write here sets the value of pins configured as outputs.
            // The spec allows masked access from 0x000 to 0x3FC. A write to GPIODATA[mask]
            // only affects the bits where the mask is 1.
            // Example: Writing to address 0x008 (mask=2) only affects pin 1.
            0x000..=0x3FC => {
                let mask = (offset >> 2) as u8;
                // Apply the write only to pins that are configured as outputs.
                let effective_mask = mask & self.direction;
                // Clear the bits we are about to set.
                self.data &= !effective_mask;
                // Set the new values.
                self.data |= byte_value & effective_mask;
            }

            // Direction register.
            GPIODIR => self.direction = byte_value,

            // Interrupt and AFSEL registers.
            GPIOIE => self.interrupt_enable = byte_value,
            GPIOAFSEL => self.afsel = byte_value,

            // Writing to the interrupt clear register acknowledges the write but does nothing.
            GPIOIC => { /* Acknowledge write, do nothing */ }

            // Ignore writes to other stubbed or read-only registers.
            _ => { /* Do nothing */ }
        }

        Ok(())
    }

    /// Resets the GPIO device to its default state.
    fn reset(&mut self) {
        self.data = 0;
        self.direction = 0;
        self.interrupt_enable = 0;
        self.afsel = 0;
    }

    /// Returns the size of the MMIO region for this device.
    fn get_size(&self) -> u64 {
        0x1000 // PL061 occupies a 4KB memory region
    }
}
