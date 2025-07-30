use crate::MmioDevice;
use crate::MmioError;
use crate::devices::register::{Register, RoRegister, RwRegister};
use std::collections::VecDeque;

/// UART register offsets (NS16550A-compatible)
const UART_RBR_THR: u64 = 0x00; // Receiver Buffer/Transmitter Holding Register
const UART_IER: u64 = 0x01; // Interrupt Enable Register
const UART_IIR_FCR: u64 = 0x02; // Interrupt Identification/FIFO Control Register
const UART_LCR: u64 = 0x03; // Line Control Register
const UART_MCR: u64 = 0x04; // Modem Control Register
const UART_LSR: u64 = 0x05; // Line Status Register
const UART_MSR: u64 = 0x06; // Modem Status Register
const UART_SCR: u64 = 0x07; // Scratch Register

/// UART Line Status Register bits
const LSR_DATA_READY: u64 = 1 << 0; // Data available in receive buffer
const LSR_OVERRUN_ERROR: u64 = 1 << 1; // Overrun error
const LSR_PARITY_ERROR: u64 = 1 << 2; // Parity error
const LSR_FRAMING_ERROR: u64 = 1 << 3; // Framing error
const LSR_BREAK_INTERRUPT: u64 = 1 << 4; // Break interrupt
const LSR_THR_EMPTY: u64 = 1 << 5; // Transmitter holding register empty
const LSR_TRANSMITTER_EMPTY: u64 = 1 << 6; // Transmitter empty
const LSR_ERROR_IN_FIFO: u64 = 1 << 7; // Error in received FIFO

/// UART device state machine
pub struct UartDevice {
    // Data FIFOs
    rx_fifo: VecDeque<u8>,
    tx_fifo: VecDeque<u8>,

    // Register state
    ier: RwRegister, // Interrupt Enable Register
    lcr: RwRegister, // Line Control Register
    mcr: RwRegister, // Modem Control Register
    lsr: RoRegister, // Line Status Register
    msr: RoRegister, // Modem Status Register
    scr: RwRegister, // Scratch Register

    // FIFO configuration
    fifo_enabled: bool,
    rx_fifo_size: usize,
    tx_fifo_size: usize,

    // Output callback for transmitted data
    output_handler: Option<Box<dyn FnMut(u8) + Send>>,
}

impl Default for UartDevice {
    fn default() -> Self {
        let mut uart = Self {
            rx_fifo: VecDeque::new(),
            tx_fifo: VecDeque::new(),
            ier: RwRegister::new(0, 0xFF),
            lcr: RwRegister::new(0, 0xFF),
            mcr: RwRegister::new(0, 0xFF),
            lsr: RoRegister::new(LSR_THR_EMPTY | LSR_TRANSMITTER_EMPTY),
            msr: RoRegister::new(0),
            scr: RwRegister::new(0, 0xFF),
            fifo_enabled: false,
            rx_fifo_size: 1,
            tx_fifo_size: 1,
            output_handler: None,
        };
        uart.update_status();
        uart
    }
}

impl UartDevice {
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

    /// Process pending transmissions
    pub fn process_tx(&mut self) {
        while let Some(data) = self.tx_fifo.pop_front() {
            if let Some(ref mut handler) = self.output_handler {
                handler(data);
            }
        }
        self.update_status();
    }

    fn update_status(&mut self) {
        let mut lsr_value = 0;

        // Set data ready bit if RX FIFO has data
        if !self.rx_fifo.is_empty() {
            lsr_value |= LSR_DATA_READY;
        }

        // Set transmitter holding register empty if TX FIFO has space
        if self.tx_fifo.len() < self.tx_fifo_size {
            lsr_value |= LSR_THR_EMPTY;
        }

        // Set transmitter empty if TX FIFO is completely empty
        if self.tx_fifo.is_empty() {
            lsr_value |= LSR_TRANSMITTER_EMPTY;
        }

        self.lsr.set_value(lsr_value);
    }

    fn read_rbr(&mut self) -> u64 {
        let data = self.rx_fifo.pop_front().unwrap_or(0);
        self.update_status();
        data as u64
    }

    fn write_thr(&mut self, value: u8) {
        if self.tx_fifo.len() < self.tx_fifo_size {
            self.tx_fifo.push_back(value);
        }
        self.update_status();
    }

    fn write_fcr(&mut self, value: u64) {
        // FIFO Control Register
        if value & 0x01 != 0 {
            self.fifo_enabled = true;
            self.rx_fifo_size = 16;
            self.tx_fifo_size = 16;
        } else {
            self.fifo_enabled = false;
            self.rx_fifo_size = 1;
            self.tx_fifo_size = 1;
        }

        // Clear FIFOs if requested
        if value & 0x02 != 0 {
            self.rx_fifo.clear();
        }
        if value & 0x04 != 0 {
            self.tx_fifo.clear();
        }

        self.update_status();
    }

    fn read_iir(&self) -> u64 {
        // Interrupt Identification Register
        // For simplicity, return "no interrupt pending"
        0x01
    }
}

impl MmioDevice for UartDevice {
    fn read(&mut self, offset: u64, size: usize) -> Result<u64, MmioError> {
        if size != 1 {
            return Err(MmioError::InvalidSize { size });
        }

        let value = match offset {
            UART_RBR_THR => self.read_rbr(),
            UART_IER => self.ier.read(),
            UART_IIR_FCR => self.read_iir(),
            UART_LCR => self.lcr.read(),
            UART_MCR => self.mcr.read(),
            UART_LSR => self.lsr.read(),
            UART_MSR => self.msr.read(),
            UART_SCR => self.scr.read(),
            _ => return Err(MmioError::UnmappedAccess(offset)),
        };

        Ok(value)
    }

    fn write(&mut self, offset: u64, size: usize, value: u64) -> Result<(), MmioError> {
        if size != 1 {
            return Err(MmioError::InvalidSize { size });
        }

        match offset {
            UART_RBR_THR => {
                self.write_thr(value as u8);
            }
            UART_IER => {
                self.ier.write(value, size)?;
            }
            UART_IIR_FCR => {
                self.write_fcr(value);
            }
            UART_LCR => {
                self.lcr.write(value, size)?;
            }
            UART_MCR => {
                self.mcr.write(value, size)?;
            }
            UART_LSR => {
                // LSR is read-only, ignore writes
            }
            UART_MSR => {
                // MSR is read-only, ignore writes
            }
            UART_SCR => {
                self.scr.write(value, size)?;
            }
            _ => return Err(MmioError::UnmappedAccess(offset)),
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.rx_fifo.clear();
        self.tx_fifo.clear();
        self.ier.reset();
        self.lcr.reset();
        self.mcr.reset();
        self.scr.reset();
        self.fifo_enabled = false;
        self.rx_fifo_size = 1;
        self.tx_fifo_size = 1;
        self.update_status();
    }

    fn get_size(&self) -> u64 {
        8 // 8 bytes for UART registers
    }
}

// Example usage:
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_uart_basic_operation() {
        let output_buffer = Arc::new(Mutex::new(Vec::new()));
        let output_buffer_clone = output_buffer.clone();

        let mut uart = UartDevice::default();
        uart.set_output_handler(move |data| {
            output_buffer_clone.lock().unwrap().push(data);
        });

        // Test writing data (transmission)
        uart.write(UART_RBR_THR, 1, b'H' as u64).unwrap();
        uart.process_tx();

        assert_eq!(*output_buffer.lock().unwrap(), vec![b'H']);

        // Test reading status
        let lsr = uart.read(UART_LSR, 1).unwrap();
        assert!(lsr & LSR_THR_EMPTY != 0);

        // Test receiving data
        uart.input_data(b'W');
        let lsr = uart.read(UART_LSR, 1).unwrap();
        assert!(lsr & LSR_DATA_READY != 0);

        let received = uart.read(UART_RBR_THR, 1).unwrap();
        assert_eq!(received, b'W' as u64);
    }
}
