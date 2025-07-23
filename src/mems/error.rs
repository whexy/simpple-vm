use anyhow::Context;
use thiserror::Error;

/// Custom error type for memory access violations using thiserror
#[derive(Error, Debug, Clone)]
pub enum MemoryError {
    #[error("Segmentation fault at address 0x{address:x} (size: {size}): {message}")]
    SegmentationFault {
        address: usize,
        size: usize,
        message: String,
    },

    #[error("Invalid memory alignment: address 0x{address:x} not aligned to {alignment} bytes")]
    InvalidAlignment { address: usize, alignment: usize },

    #[error("Memory region overlap: 0x{start:x}-0x{end:x} overlaps with existing segment")]
    RegionOverlap { start: usize, end: usize },

    #[error("Invalid size: {size} bytes is invalid for this operation")]
    InvalidSize { size: usize },

    #[error("Out of memory: cannot allocate {requested} bytes")]
    OutOfMemory { requested: usize },
}

impl MemoryError {
    /// Create a segmentation fault error
    pub fn segfault(address: usize, size: usize, message: impl Into<String>) -> Self {
        Self::SegmentationFault {
            address,
            size,
            message: message.into(),
        }
    }

    /// Create an invalid alignment error
    pub fn invalid_alignment(address: usize, alignment: usize) -> Self {
        Self::InvalidAlignment { address, alignment }
    }

    /// Create a region overlap error
    pub fn region_overlap(start: usize, end: usize) -> Self {
        Self::RegionOverlap { start, end }
    }

    /// Create an invalid size error
    pub fn invalid_size(size: usize) -> Self {
        Self::InvalidSize { size }
    }

    /// Create an out of memory error
    pub fn out_of_memory(requested: usize) -> Self {
        Self::OutOfMemory { requested }
    }
}

/// Result type alias for memory operations using anyhow
pub type MemoryResult<T> = anyhow::Result<T>;

/// Convenience function to create a segmentation fault error with anyhow context
pub fn segfault_error(address: usize, size: usize, message: impl Into<String>) -> anyhow::Error {
    MemoryError::segfault(address, size, message).into()
}

/// Convenience function to create an invalid alignment error with anyhow context
pub fn alignment_error(address: usize, alignment: usize) -> anyhow::Error {
    MemoryError::invalid_alignment(address, alignment).into()
}

/// Extension trait for Result to add memory-specific context
pub trait MemoryResultExt<T> {
    /// Add context about a memory operation
    fn with_memory_context(self, operation: &str, address: usize) -> MemoryResult<T>;

    /// Add context about a memory range operation
    fn with_range_context(self, operation: &str, address: usize, size: usize) -> MemoryResult<T>;
}

impl<T> MemoryResultExt<T> for Result<T, MemoryError> {
    fn with_memory_context(self, operation: &str, address: usize) -> MemoryResult<T> {
        self.map_err(|e| {
            anyhow::Error::from(e).context(format!(
                "Memory operation '{operation}' failed at address 0x{address:x}"
            ))
        })
    }

    fn with_range_context(self, operation: &str, address: usize, size: usize) -> MemoryResult<T> {
        self.map_err(|e| {
            anyhow::Error::from(e).context(format!(
                "Memory operation '{}' failed for range 0x{:x}-0x{:x} (size: {})",
                operation,
                address,
                address.saturating_add(size).saturating_sub(1),
                size
            ))
        })
    }
}

impl<T> MemoryResultExt<T> for Result<T, anyhow::Error> {
    fn with_memory_context(self, operation: &str, address: usize) -> MemoryResult<T> {
        self.with_context(|| {
            format!("Memory operation '{operation}' failed at address 0x{address:x}")
        })
    }

    fn with_range_context(self, operation: &str, address: usize, size: usize) -> MemoryResult<T> {
        self.with_context(|| {
            format!(
                "Memory operation '{}' failed for range 0x{:x}-0x{:x} (size: {})",
                operation,
                address,
                address.saturating_add(size).saturating_sub(1),
                size
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_error_creation() {
        let err = MemoryError::segfault(0x1000, 4, "Invalid access");
        assert!(err.to_string().contains("0x1000"));
        assert!(err.to_string().contains("size: 4"));
        assert!(err.to_string().contains("Invalid access"));
    }

    #[test]
    fn test_anyhow_integration() {
        let result: MemoryResult<i32> = Err(MemoryError::segfault(0x2000, 8, "Test error").into());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("0x2000"));
    }

    #[test]
    fn test_memory_result_ext() {
        let result: Result<(), MemoryError> = Err(MemoryError::segfault(0x1000, 4, "Test"));

        let memory_result = result.with_memory_context("read", 0x1000);
        assert!(memory_result.is_err());
        let err_string = memory_result.unwrap_err().to_string();
        assert!(err_string.contains("Memory operation 'read' failed"));
        assert!(err_string.contains("0x1000"));
    }

    #[test]
    fn test_convenience_functions() {
        let err = segfault_error(0x3000, 16, "Test segfault");
        assert!(err.to_string().contains("0x3000"));

        let align_err = alignment_error(0x1001, 4);
        assert!(align_err.to_string().contains("0x1001"));
        assert!(align_err.to_string().contains("4 bytes"));
    }
}
