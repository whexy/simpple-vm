use crate::MmioDevice;
use crate::MmioError;
use std::collections::VecDeque;

// --- ARM PL011 Register Offsets ---
// Note: These are 4-byte (word) aligned offsets.
const UARTDR: u64 = 0x000; // Data Register
const UARTFR: u64 = 0x018; // Flag Register
const UARTLCR_H: u64 = 0x02C; // Line Control Register
const UARTCR: u64 = 0x030; // Control Register
const UARTIMSC: u64 = 0x038; // Interrupt Mask Set/Clear Register
const UARTICR: u64 = 0x044; // Interrupt Clear Register
const UART_PERIPH_ID_BASE: u64 = 0xFE0; // Start of Peripheral ID registers

// --- Flag Register (UARTFR) bits ---
const FLAG_TXFE: u32 = 1 << 7; // Transmit FIFO empty
const FLAG_RXFF: u32 = 1 << 6; // Receive FIFO full
const FLAG_TXFF: u32 = 1 << 5; // Transmit FIFO full
const FLAG_RXFE: u32 = 1 << 4; // Receive FIFO empty

// --- Line Control Register (UARTLCR_H) bits ---
const LCR_H_FEN: u32 = 1 << 4; // FIFO Enable

// --- Control Register (UARTCR) bits ---
const CR_RXE: u32 = 1 << 9; // Receive Enable
const CR_TXE: u32 = 1 << 8; // Transmit Enable
const CR_UARTEN: u32 = 1 << 0; // UART Enable

// Default FIFO size when enabled, from QEMU's implementation.
const PL011_FIFO_DEPTH: usize = 16;

/// ARM PL011 UART device state machine (minimal implementation)
pub struct Pl011Device {
    // Data FIFOs
    rx_fifo: VecDeque<u8>,
    tx_fifo: VecDeque<u8>,

    // Register state (using simple u32 for word-sized registers)
    flags: u32, // Flag Register (Read-Only)
    lcr_h: u32, // Line Control Register
    cr: u32,    // Control Register
    imsc: u32,  // Interrupt Mask

    // Peripheral ID registers
    id: [u8; 8],

    // FIFO configuration
    fifo_enabled: bool,
    rx_fifo_size: usize,
    tx_fifo_size: usize,

    // Output callback for transmitted data
    output_handler: Option<Box<dyn FnMut(u8) + Send>>,
}

impl Default for Pl011Device {
    fn default() -> Self {
        let mut uart = Self {
            rx_fifo: VecDeque::new(),
            tx_fifo: VecDeque::new(),

            // Initialize registers to match QEMU's reset state
            flags: FLAG_TXFE | FLAG_RXFE, // TX and RX FIFOs are empty
            lcr_h: 0,
            cr: CR_TXE | CR_RXE, // U-Boot expects TX/RX to be enabled
            imsc: 0,

            // Standard ARM PL011 Peripheral ID
            id: [0x11, 0x10, 0x14, 0x00, 0x0d, 0xf0, 0x05, 0xb1],

            fifo_enabled: false,
            rx_fifo_size: 1,
            tx_fifo_size: 1,
            output_handler: None,
        };
        uart.update_status();
        uart
    }
}

impl Pl011Device {
    pub fn set_output_handler<F>(&mut self, handler: F)
    where
        F: FnMut(u8) + Send + 'static,
    {
        self.output_handler = Some(Box::new(handler));
    }

    /// Input data to the UART (simulates receiving data)
    pub fn input_data(&mut self, data: u8) {
        if self.rx_fifo.len() < self.rx_fifo_size {
            self.rx_fifo.push_back(data);
        }
        self.update_status();
    }

    /// The PL011 status is maintained in the `flags` register.
    fn update_status(&mut self) {
        // Clear status bits
        self.flags &= !(FLAG_RXFE | FLAG_RXFF | FLAG_TXFE | FLAG_TXFF);

        if self.rx_fifo.is_empty() {
            self.flags |= FLAG_RXFE; // Receive FIFO empty
        }
        if self.rx_fifo.len() >= self.rx_fifo_size {
            self.flags |= FLAG_RXFF; // Receive FIFO full
        }

        if self.tx_fifo.is_empty() {
            self.flags |= FLAG_TXFE; // Transmit FIFO empty
        }
        if self.tx_fifo.len() >= self.tx_fifo_size {
            self.flags |= FLAG_TXFF; // Transmit FIFO full
        }
    }

    fn read_dr(&mut self) -> u64 {
        let data = self.rx_fifo.pop_front().unwrap_or(0);
        self.update_status();
        data as u64
    }

    fn write_dr(&mut self, value: u8) {
        // Check if UART and transmitter are enabled
        if (self.cr & (CR_UARTEN | CR_TXE)) != (CR_UARTEN | CR_TXE) {
            // Silently ignore write if UART/TX is disabled, like real hardware.
            return;
        }

        if self.tx_fifo.len() < self.tx_fifo_size {
            self.tx_fifo.push_back(value);
            // For simplicity, we immediately "transmit" the character.
            if let Some(ref mut handler) = self.output_handler {
                handler(value);
                self.tx_fifo.pop_front(); // Immediately sent
            }
        }
        self.update_status();
    }

    fn write_lcr_h(&mut self, value: u32) {
        self.lcr_h = value;
        let fifo_just_enabled = (value & LCR_H_FEN) != 0;

        if fifo_just_enabled != self.fifo_enabled {
            self.fifo_enabled = fifo_just_enabled;
            if self.fifo_enabled {
                self.rx_fifo_size = PL011_FIFO_DEPTH;
                self.tx_fifo_size = PL011_FIFO_DEPTH;
            } else {
                self.rx_fifo_size = 1;
                self.tx_fifo_size = 1;
            }
            // Real hardware would reset FIFOs here, so we do too.
            self.rx_fifo.clear();
            self.tx_fifo.clear();
            self.update_status();
        }
    }
}

impl MmioDevice for Pl011Device {
    fn read(&mut self, offset: u64, size: usize) -> Result<u64, MmioError> {
        // PL011 has 4-byte registers
        if size != 4 {
            return Err(MmioError::InvalidSize { size });
        }

        let value = match offset {
            UARTDR => self.read_dr(),
            UARTFR => self.flags as u64,
            UARTLCR_H => self.lcr_h as u64,
            UARTCR => self.cr as u64,
            UARTIMSC => self.imsc as u64,

            // Stub other common registers to prevent unmapped access errors
            0x028 => 0, // UARTFBRD (Fractional Baud Rate)
            0x024 => 0, // UARTIBRD (Integer Baud Rate)
            0x03C => 0, // UARTRIS (Raw Interrupt Status)
            0x040 => 0, // UARTMIS (Masked Interrupt Status)

            // Peripheral ID registers
            UART_PERIPH_ID_BASE..=0xFFC => {
                let index = ((offset - UART_PERIPH_ID_BASE) / 4) as usize;
                if index < self.id.len() {
                    self.id[index] as u64
                } else {
                    0
                }
            }

            _ => return Err(MmioError::UnmappedAccess(offset)),
        };

        Ok(value)
    }

    fn write(&mut self, offset: u64, size: usize, value: u64) -> Result<(), MmioError> {
        if size != 4 {
            return Err(MmioError::InvalidSize { size });
        }

        match offset {
            UARTDR => self.write_dr(value as u8),
            UARTLCR_H => self.write_lcr_h(value as u32),
            UARTCR => self.cr = value as u32,
            UARTIMSC => self.imsc = value as u32,

            // On write, clear the specified interrupt flags from the (unimplemented) level
            UARTICR => { /* Acknowledge write, do nothing */ }

            // Ignore writes to read-only or stubbed registers
            UARTFR => { /* Read Only */ }
            0x028 | 0x024 => { /* Stubbed */ }

            _ => return Err(MmioError::UnmappedAccess(offset)),
        }

        Ok(())
    }

    fn reset(&mut self) {
        *self = Pl011Device::default();
    }

    fn get_size(&self) -> u64 {
        0x1000 // PL011 occupies a 4KB memory region
    }
}
