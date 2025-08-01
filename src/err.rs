use ahvf::HypervisorError;
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

    #[error("MMIO error: {0}")]
    MMIO(#[from] MmioError),

    #[error("General error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("System register not found: {0}")]
    SysRegNotFound(String),
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

#[derive(Error, Debug, Clone)]
pub enum MmioError {
    #[error("Unmapped memory access at address 0x{0:016x}")]
    UnmappedAccess(u64),

    #[error("Invalid alignment: address 0x{addr:016x} not aligned for {size}-byte access")]
    InvalidAlignment { addr: u64, size: usize },

    #[error("Invalid access size: {size} bytes (must be 1, 2, 4, or 8)")]
    InvalidSize { size: usize },

    #[error("Device error: {0}")]
    DeviceError(String),

    #[error(
        "Overlapping MMIO region: new region [0x{new_start:016x}, 0x{new_end:016x}) overlaps with existing region [0x{existing_start:016x}, 0x{existing_end:016x})"
    )]
    OverlappingRegion {
        existing_start: u64,
        existing_end: u64,
        new_start: u64,
        new_end: u64,
    },
}

// Helper constructor for the overlapping region error
impl MmioError {
    pub fn overlapping_region(existing: (u64, u64), new: (u64, u64)) -> Self {
        Self::OverlappingRegion {
            existing_start: existing.0,
            existing_end: existing.1,
            new_start: new.0,
            new_end: new.1,
        }
    }
}
