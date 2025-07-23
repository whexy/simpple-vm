use crate::error::{MemoryError, MemoryResult, MemoryResultExt};
use anyhow::Context;

// Shared memory management
#[derive(Debug)]
pub struct Segment {
    pub base: usize,       // base address (guest physical)
    pub size: usize,       // size
    pub memory: Box<[u8]>, // memory content
}

impl Segment {
    pub fn new(base: usize, size: usize) -> Self {
        Segment {
            base,
            size,
            memory: vec![0; size].into_boxed_slice(),
        }
    }

    pub fn contains(&self, address: usize, size: usize) -> bool {
        address >= self.base && address.saturating_add(size) <= self.base.saturating_add(self.size)
    }

    // Get offset within segment for given address
    pub fn get_offset(&self, address: usize) -> Option<usize> {
        if address >= self.base && address < self.base.saturating_add(self.size) {
            Some(address - self.base)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct SharedMemory {
    pub segments: Vec<Segment>, // list of segments
}

impl SharedMemory {
    pub fn add_segment(&mut self, base: usize, size: usize) -> MemoryResult<&mut Segment> {
        // Check for overlaps with existing segments
        for segment in &self.segments {
            let end = base.saturating_add(size);
            let seg_end = segment.base.saturating_add(segment.size);

            if (base >= segment.base && base < seg_end)
                || (end > segment.base && end <= seg_end)
                || (base <= segment.base && end >= seg_end)
            {
                return Err(MemoryError::region_overlap(base, end).into());
            }
        }

        if size == 0 {
            return Err(MemoryError::invalid_size(size).into());
        }

        let segment = Segment::new(base, size);
        self.segments.push(segment);
        Ok(self.segments.last_mut().unwrap())
    }

    // Find segment containing the address range
    fn find_segment(&self, address: usize, size: usize) -> Result<&Segment, MemoryError> {
        self.segments
            .iter()
            .find(|seg| seg.contains(address, size))
            .ok_or_else(|| {
                MemoryError::segfault(
                    address,
                    size,
                    format!(
                        "Address range 0x{:x}-0x{:x} not mapped",
                        address,
                        address.saturating_add(size).saturating_sub(1)
                    ),
                )
            })
    }

    // Find mutable segment containing the address range
    fn find_segment_mut(
        &mut self,
        address: usize,
        size: usize,
    ) -> Result<&mut Segment, MemoryError> {
        self.segments
            .iter_mut()
            .find(|seg| seg.contains(address, size))
            .ok_or_else(|| {
                MemoryError::segfault(
                    address,
                    size,
                    format!(
                        "Address range 0x{:x}-0x{:x} not mapped",
                        address,
                        address.saturating_add(size).saturating_sub(1)
                    ),
                )
            })
    }

    // Raw byte operations
    pub fn read_bytes(&self, address: usize, size: usize) -> MemoryResult<Vec<u8>> {
        if size == 0 {
            return Ok(Vec::new());
        }

        let segment =
            self.find_segment(address, size)
                .with_range_context("read_bytes", address, size)?;

        let offset = segment.get_offset(address).unwrap();
        Ok(segment.memory[offset..offset + size].to_vec())
    }

    pub fn write_bytes(&mut self, address: usize, data: &[u8]) -> MemoryResult<()> {
        let size = data.len();
        if size == 0 {
            return Ok(());
        }

        let segment = self.find_segment_mut(address, size).with_range_context(
            "write_bytes",
            address,
            size,
        )?;

        let offset = segment.get_offset(address).unwrap();
        segment.memory[offset..offset + size].copy_from_slice(data);
        Ok(())
    }

    // Generic read/write for any sized integer type
    pub fn read<T>(&self, address: usize) -> MemoryResult<T>
    where
        T: FromBytes,
    {
        let size = std::mem::size_of::<T>();
        let bytes = self.read_bytes(address, size).with_context(|| {
            format!(
                "Failed to read {} at address 0x{:x}",
                std::any::type_name::<T>(),
                address
            )
        })?;
        Ok(T::from_le_bytes(&bytes))
    }

    pub fn write<T>(&mut self, address: usize, value: T) -> MemoryResult<()>
    where
        T: ToBytes,
    {
        self.write_bytes(address, &value.to_le_bytes())
            .with_context(|| {
                format!(
                    "Failed to write {} at address 0x{:x}",
                    std::any::type_name::<T>(),
                    address
                )
            })
    }

    // Aligned read operations (useful for performance-critical code)
    pub fn read_aligned<T>(&self, address: usize) -> MemoryResult<T>
    where
        T: FromBytes,
    {
        let alignment = std::mem::align_of::<T>();
        if address % alignment != 0 {
            return Err(MemoryError::invalid_alignment(address, alignment).into());
        }
        self.read(address)
    }

    pub fn write_aligned<T>(&mut self, address: usize, value: T) -> MemoryResult<()>
    where
        T: ToBytes,
    {
        let alignment = std::mem::align_of::<T>();
        if address % alignment != 0 {
            return Err(MemoryError::invalid_alignment(address, alignment).into());
        }
        self.write(address, value)
    }

    // Bulk operations
    pub fn read_array<T>(&self, address: usize, count: usize) -> MemoryResult<Vec<T>>
    where
        T: FromBytes,
    {
        let element_size = std::mem::size_of::<T>();
        let total_size = element_size
            .checked_mul(count)
            .ok_or_else(|| MemoryError::invalid_size(count))?;

        let bytes = self.read_bytes(address, total_size).with_context(|| {
            format!(
                "Failed to read array of {} {} elements at 0x{:x}",
                count,
                std::any::type_name::<T>(),
                address
            )
        })?;

        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let start = i * element_size;
            let end = start + element_size;
            result.push(T::from_le_bytes(&bytes[start..end]));
        }
        Ok(result)
    }

    pub fn write_array<T>(&mut self, address: usize, values: &[T]) -> MemoryResult<()>
    where
        T: ToBytes + Clone,
    {
        let mut bytes = Vec::new();
        for value in values {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        self.write_bytes(address, &bytes).with_context(|| {
            format!(
                "Failed to write array of {} {} elements at 0x{:x}",
                values.len(),
                std::any::type_name::<T>(),
                address
            )
        })
    }
}

// Traits for generic type handling
pub trait FromBytes: Sized {
    fn from_le_bytes(bytes: &[u8]) -> Self;
}

pub trait ToBytes {
    fn to_le_bytes(&self) -> Vec<u8>;
}

// Implement traits for standard integer types
impl FromBytes for u8 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}

impl FromBytes for u16 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u16::from_le_bytes([bytes[0], bytes[1]])
    }
}

impl FromBytes for u32 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }
}

impl FromBytes for u64 {
    fn from_le_bytes(bytes: &[u8]) -> Self {
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }
}

impl ToBytes for u8 {
    fn to_le_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl ToBytes for u16 {
    fn to_le_bytes(&self) -> Vec<u8> {
        (*self).to_le_bytes().to_vec()
    }
}

impl ToBytes for u32 {
    fn to_le_bytes(&self) -> Vec<u8> {
        (*self).to_le_bytes().to_vec()
    }
}

impl ToBytes for u64 {
    fn to_le_bytes(&self) -> Vec<u8> {
        (*self).to_le_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut memory = SharedMemory::default();
        memory.add_segment(0x1000, 0x1000).unwrap();

        // Test u32 write/read
        memory.write::<u32>(0x1000, 0x12345678).unwrap();
        assert_eq!(memory.read::<u32>(0x1000).unwrap(), 0x12345678);

        // Test u64 write/read
        memory.write::<u64>(0x1008, 0x123456789ABCDEF0).unwrap();
        assert_eq!(memory.read::<u64>(0x1008).unwrap(), 0x123456789ABCDEF0);

        // Test segmentation fault
        assert!(memory.read::<u32>(0x2000).is_err());
        assert!(memory.write::<u32>(0x2000, 42).is_err());
    }

    #[test]
    fn test_error_types() {
        let mut memory = SharedMemory::default();
        memory.add_segment(0x1000, 0x1000).unwrap();

        // Test overlap detection
        let overlap_result = memory.add_segment(0x1500, 0x1000);
        assert!(overlap_result.is_err());

        // Test alignment
        let align_result = memory.read_aligned::<u32>(0x1001);
        assert!(align_result.is_err());
    }

    #[test]
    fn test_array_operations() {
        let mut memory = SharedMemory::default();
        memory.add_segment(0x1000, 0x1000).unwrap();

        let values = vec![1u32, 2, 3, 4, 5];
        memory.write_array(0x1000, &values).unwrap();

        let read_values = memory.read_array::<u32>(0x1000, 5).unwrap();
        assert_eq!(values, read_values);
    }
}
