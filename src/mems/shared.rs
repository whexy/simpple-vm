use crate::SimppleError;
use crate::err::MemoryError;
use anyhow::{Context, Result};

// Shared memory management
#[derive(Debug)]
struct Segment {
    base: u64,                     // base address (guest physical)
    size: usize,                   // size
    handle: ahv::AllocationHandle, // handle to the memory allocator
}

impl Segment {
    pub fn new(handle: ahv::AllocationHandle, base: u64, size: usize) -> Self {
        Segment { base, size, handle }
    }

    pub fn contains(&self, address: u64, size: usize) -> bool {
        address >= self.base
            && address.saturating_add(size as u64) <= self.base.saturating_add(self.size as u64)
    }

    // Get offset within segment for given address
    pub fn get_offset(&self, address: u64) -> Option<u64> {
        if address >= self.base && address < self.base.saturating_add(self.size as u64) {
            Some(address - self.base)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub struct SharedMemory {
    segments: Vec<Segment>, // list of segments
}

impl SharedMemory {
    pub fn add_segment(
        &mut self,
        vm: &mut ahv::VirtualMachine,
        base: u64,
        size: usize,
        permission: ahv::MemoryPermission,
    ) -> Result<(), SimppleError> {
        // Check for overlaps with existing segments
        for segment in &self.segments {
            let end = base.saturating_add(size as u64);
            let seg_end = segment.base.saturating_add(segment.size as u64);

            if (base >= segment.base && base < seg_end)
                || (end > segment.base && end <= seg_end)
                || (base <= segment.base && end >= seg_end)
            {
                return Err(MemoryError::region_overlap(base, end).into());
            }
        }

        let handle = vm.allocate(size)?;
        vm.map(handle, base, permission)?;

        let segment = Segment::new(handle, base, size);
        self.segments.push(segment);
        Ok(())
    }

    // Find segment containing the address range
    fn find_segment(&self, address: u64, size: usize) -> Result<&Segment, MemoryError> {
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
                        address.saturating_add(size as u64).saturating_sub(1)
                    ),
                )
            })
    }

    // Raw byte operations
    pub fn read_bytes(
        &self,
        vm: &ahv::VirtualMachine,
        address: u64,
        size: usize,
    ) -> Result<Vec<u8>, SimppleError> {
        if size == 0 {
            return Ok(Vec::new());
        }

        let segment = self.find_segment(address, size)?;
        let offset = segment.get_offset(address).unwrap() as usize;

        let memory = vm.get_allocation_slice(segment.handle)?;

        Ok(memory[offset..offset + size].to_vec())
    }

    pub fn write_bytes(
        &self,
        vm: &mut ahv::VirtualMachine,
        address: u64,
        data: &[u8],
    ) -> Result<(), SimppleError> {
        let size = data.len();
        if size == 0 {
            return Ok(());
        }

        let segment = self.find_segment(address, size)?;
        let offset = segment.get_offset(address).unwrap() as usize;

        let memory = vm.get_allocation_slice_mut(segment.handle)?;
        memory[offset..offset + size].copy_from_slice(data);
        Ok(())
    }

    // Generic read/write for any sized integer type
    pub fn read<T>(&self, vm: &ahv::VirtualMachine, address: u64) -> Result<T>
    where
        T: FromBytes,
    {
        let size = std::mem::size_of::<T>();
        let bytes = self.read_bytes(vm, address, size).with_context(|| {
            format!(
                "Failed to read {} at address 0x{:x}",
                std::any::type_name::<T>(),
                address
            )
        })?;
        Ok(T::from_le_bytes(&bytes))
    }

    pub fn write<T>(&self, vm: &mut ahv::VirtualMachine, address: u64, value: T) -> Result<()>
    where
        T: ToBytes,
    {
        self.write_bytes(vm, address, &value.to_le_bytes())
            .with_context(|| {
                format!(
                    "Failed to write {} at address 0x{:x}",
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
