use ahv::HypervisorError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimppleError {
    #[error("Hypervisor error: {0:?}")]
    Hypervisor(HypervisorError),

    #[error("Keystone assembly error: {0}")]
    Keystone(#[from] keystone_engine::KeystoneError),

    #[error("Capstone disassembly error: {0}")]
    Capstone(#[from] capstone::Error),

    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("General error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<HypervisorError> for SimppleError {
    fn from(err: HypervisorError) -> Self {
        SimppleError::Hypervisor(err)
    }
}

#[derive(Error, Debug, Clone)]
pub enum MemoryError {
    #[error("Segmentation fault at address 0x{address:x} (size: {size}): {message}")]
    SegmentationFault {
        address: u64,
        size: usize,
        message: String,
    },

    #[error("Memory region overlap: 0x{start:x}-0x{end:x} overlaps with existing segment")]
    RegionOverlap { start: u64, end: u64 },

    #[error("Invalid size: {size} bytes is invalid for this operation")]
    InvalidSize { size: usize },
}

impl MemoryError {
    pub fn segfault(address: u64, size: usize, message: impl Into<String>) -> Self {
        Self::SegmentationFault {
            address,
            size,
            message: message.into(),
        }
    }

    pub fn region_overlap(start: u64, end: u64) -> Self {
        Self::RegionOverlap { start, end }
    }

    pub fn invalid_size(size: usize) -> Self {
        Self::InvalidSize { size }
    }
}
