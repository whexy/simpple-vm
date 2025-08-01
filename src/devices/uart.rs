use crate::devices::MmioDevice;
use crate::err::MmioError;
use std::collections::VecDeque;
use std::io::{self, Write};

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

// Standard ARM PL011 Peripheral ID
const PL011_PERIPHERAL_ID: [u8; 8] = [0x11, 0x10, 0x14, 0x00, 0x0d, 0xf0, 0x05, 0xb1];

/// ARM PL011 UART device state machine (generic over output interface)
pub struct Pl011Device<W: Write> {
    // Data FIFOs
    rx_fifo: VecDeque<u8>,
    tx_fifo: VecDeque<u8>,

    // Register state (using simple u32 for word-sized registers)
    flags: u32, // Flag Register (Read-Only)
    lcr_h: u32, // Line Control Register
    cr: u32,    // Control Register
    imsc: u32,  // Interrupt Mask

    // FIFO configuration
    fifo_enabled: bool,
    rx_fifo_size: usize,
    tx_fifo_size: usize,

    // Line buffering for output
    line_buffer: Vec<u8>,

    // Generic output interface
    output: W,
}

impl<W: Write> Pl011Device<W> {
    /// Creates a new PL011 device with the specified output interface
    pub fn new(output: W) -> Self {
        let mut uart = Self {
            rx_fifo: VecDeque::new(),
            tx_fifo: VecDeque::new(),

            // Initialize registers to match QEMU's reset state
            flags: FLAG_TXFE | FLAG_RXFE, // TX and RX FIFOs are empty
            lcr_h: 0,
            cr: CR_TXE | CR_RXE, // U-Boot expects TX/RX to be enabled
            imsc: 0,

            fifo_enabled: false,
            rx_fifo_size: 1,
            tx_fifo_size: 1,
            line_buffer: Vec::new(),
            output,
        };
        uart.update_status();
        uart
    }

    /// Input data to the UART (simulates receiving data)
    pub fn input_data(&mut self, data: u8) {
        if self.rx_fifo.len() < self.rx_fifo_size {
            self.rx_fifo.push_back(data);
        }
        self.update_status();
    }

    /// Get a mutable reference to the output interface
    pub fn output_mut(&mut self) -> &mut W {
        &mut self.output
    }

    /// Get a reference to the output interface
    pub fn output(&self) -> &W {
        &self.output
    }

    /// Flush any remaining content in the line buffer to the output
    pub fn flush_line_buffer(&mut self) -> io::Result<()> {
        if !self.line_buffer.is_empty() {
            self.output.write_all(&self.line_buffer)?;
            self.output.flush()?;
            self.line_buffer.clear();
        }
        Ok(())
    }

    /// Handle transmitted character with line buffering
    fn handle_transmitted_char(&mut self, byte: u8) -> io::Result<()> {
        match byte {
            b'\n' => {
                // Line complete - print the entire line including newline
                self.line_buffer.push(byte);
                self.output.write_all(&self.line_buffer)?;
                self.output.flush()?;
                self.line_buffer.clear();
            }
            b'\r' => {
                // Carriage return - handle different line ending styles
                self.line_buffer.push(byte);
                // Don't flush yet in case this is followed by \n (CRLF)
            }
            _ => {
                // Regular character - add to line buffer
                self.line_buffer.push(byte);
            }
        }
        Ok(())
    }

    /// Updates the PL011 status flags based on FIFO states
    fn update_status(&mut self) {
        // Clear status bits
        self.flags &= !(FLAG_RXFE | FLAG_RXFF | FLAG_TXFE | FLAG_TXFF);

        // Update receive FIFO flags
        if self.rx_fifo.is_empty() {
            self.flags |= FLAG_RXFE; // Receive FIFO empty
        }
        if self.rx_fifo.len() >= self.rx_fifo_size {
            self.flags |= FLAG_RXFF; // Receive FIFO full
        }

        // Update transmit FIFO flags
        if self.tx_fifo.is_empty() {
            self.flags |= FLAG_TXFE; // Transmit FIFO empty
        }
        if self.tx_fifo.len() >= self.tx_fifo_size {
            self.flags |= FLAG_TXFF; // Transmit FIFO full
        }
    }

    /// Read from the data register (receives data)
    fn read_dr(&mut self) -> u64 {
        let data = self.rx_fifo.pop_front().unwrap_or(0);
        self.update_status();
        u64::from(data)
    }

    /// Write to the data register (transmits data)
    fn write_dr(&mut self, value: u8) {
        // Check if UART and transmitter are enabled
        if (self.cr & (CR_UARTEN | CR_TXE)) != (CR_UARTEN | CR_TXE) {
            // Silently ignore write if UART/TX is disabled, like real hardware.
            return;
        }

        if self.tx_fifo.len() < self.tx_fifo_size {
            self.tx_fifo.push_back(value);
            // For simplicity, we immediately "transmit" the character.
            // Ignore I/O errors during transmission (hardware behavior)
            let _ = self.handle_transmitted_char(value);
            self.tx_fifo.pop_front(); // Immediately sent
        }
        self.update_status();
    }

    /// Write to the line control register
    fn write_lcr_h(&mut self, value: u32) {
        self.lcr_h = value;
        let fifo_enable_requested = (value & LCR_H_FEN) != 0;

        if fifo_enable_requested != self.fifo_enabled {
            self.fifo_enabled = fifo_enable_requested;

            // Update FIFO sizes based on enable state
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

    /// Get the peripheral ID byte at the specified index
    fn get_peripheral_id_byte(&self, index: usize) -> u8 {
        PL011_PERIPHERAL_ID.get(index).copied().unwrap_or(0)
    }
}

impl<W: Write> Drop for Pl011Device<W> {
    fn drop(&mut self) {
        let _ = self.flush_line_buffer();
    }
}

impl<W: Write> MmioDevice for Pl011Device<W> {
    fn read(&mut self, offset: u64, size: usize) -> Result<u64, MmioError> {
        // PL011 has 4-byte registers
        if size != 4 {
            return Err(MmioError::InvalidSize { size });
        }

        let value = match offset {
            UARTDR => self.read_dr(),
            UARTFR => u64::from(self.flags),
            UARTLCR_H => u64::from(self.lcr_h),
            UARTCR => u64::from(self.cr),
            UARTIMSC => u64::from(self.imsc),

            // Stub other common registers to prevent unmapped access errors
            0x028 => 0, // UARTFBRD (Fractional Baud Rate)
            0x024 => 0, // UARTIBRD (Integer Baud Rate)
            0x03C => 0, // UARTRIS (Raw Interrupt Status)
            0x040 => 0, // UARTMIS (Masked Interrupt Status)

            // Peripheral ID registers
            UART_PERIPH_ID_BASE..=0xFFC => {
                let index = ((offset - UART_PERIPH_ID_BASE) / 4) as usize;
                u64::from(self.get_peripheral_id_byte(index))
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
        // We can't easily reset to default with a generic type, so we clear state manually
        self.rx_fifo.clear();
        self.tx_fifo.clear();
        self.flags = FLAG_TXFE | FLAG_RXFE;
        self.lcr_h = 0;
        self.cr = CR_TXE | CR_RXE;
        self.imsc = 0;
        self.fifo_enabled = false;
        self.rx_fifo_size = 1;
        self.tx_fifo_size = 1;
        self.line_buffer.clear();
        self.update_status();
    }

    fn get_size(&self) -> u64 {
        0x1000 // PL011 occupies a 4KB memory region
    }
}

// Type aliases for common use cases
pub type Pl011Stdout = Pl011Device<io::Stdout>;
pub type Pl011File = Pl011Device<std::fs::File>;
pub type Pl011Vec = Pl011Device<std::io::Cursor<Vec<u8>>>;

// Convenience constructors
impl Pl011Device<io::Stdout> {
    /// Create a PL011 device that outputs to stdout
    pub fn stdout() -> Self {
        Self::new(io::stdout())
    }
}

impl Pl011Device<std::fs::File> {
    /// Create a PL011 device that outputs to a file
    pub fn file<P: AsRef<std::path::Path>>(path: P) -> io::Result<Self> {
        let file = std::fs::File::create(path)?;
        Ok(Self::new(file))
    }
}

impl Pl011Device<std::io::Cursor<Vec<u8>>> {
    /// Create a PL011 device that outputs to a buffer (useful for testing)
    pub fn buffer() -> Self {
        Self::new(std::io::Cursor::new(Vec::new()))
    }

    /// Get the buffered output as a byte slice
    pub fn get_output(&self) -> &[u8] {
        self.output.get_ref()
    }

    /// Get the buffered output as a string (assuming UTF-8)
    pub fn get_output_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.output.get_ref().clone())
    }
}
