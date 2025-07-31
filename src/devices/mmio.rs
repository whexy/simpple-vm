use std::collections::BTreeMap;

use crate::err::MmioError;

pub trait MmioDevice {
    fn read(&mut self, offset: u64, size: usize) -> Result<u64, MmioError>;
    fn write(&mut self, offset: u64, size: usize, value: u64) -> Result<(), MmioError>;
    fn reset(&mut self);
    fn get_size(&self) -> u64;
}

struct MmioRegion {
    base_addr: u64,
    size: u64,
    device: Box<dyn MmioDevice>,
}

#[derive(Default)]
pub struct MmioManager {
    regions: BTreeMap<u64, MmioRegion>, // Sorted by base address
}

impl MmioManager {
    pub fn register_device(
        &mut self,
        base: u64,
        device: Box<dyn MmioDevice>,
    ) -> Result<(), MmioError> {
        let size = device.get_size();

        // Check for overlaps
        if let Some(existing) = self.find_overlap(base, size) {
            return Err(MmioError::overlapping_region(existing, (base, base + size)));
        }

        self.regions.insert(
            base,
            MmioRegion {
                base_addr: base,
                size,
                device,
            },
        );

        Ok(())
    }

    fn locate(&mut self, addr: u64, size: usize) -> Result<&mut MmioRegion, MmioError> {
        if !matches!(size, 1 | 2 | 4 | 8) {
            return Err(MmioError::InvalidSize { size });
        }
        if addr & (size as u64 - 1) != 0 {
            return Err(MmioError::InvalidAlignment { addr, size });
        }
        // Find the device
        let region = self.find_region(addr)?;
        let offset = addr - region.base_addr;

        // Ensure access is within bounds
        if offset + size as u64 > region.size {
            return Err(MmioError::UnmappedAccess(addr));
        }
        Ok(region)
    }

    pub fn handle_write(&mut self, addr: u64, size: usize, value: u64) -> Result<(), MmioError> {
        log::debug!("Write {value} to {addr:#0x} of size {size}");
        let region = self.locate(addr, size)?;
        let offset = addr - region.base_addr;
        region.device.write(offset, size, value)?;
        Ok(())
    }

    pub fn handle_read(&mut self, addr: u64, size: usize) -> Result<u64, MmioError> {
        log::debug!("Read from {addr:#0x} of size {size}");
        let region = self.locate(addr, size)?;
        let offset = addr - region.base_addr;
        region.device.read(offset, size)
    }

    fn find_region(&mut self, addr: u64) -> Result<&mut MmioRegion, MmioError> {
        // Find the region that could contain this address
        let (_, region) = self
            .regions
            .range_mut(..=addr)
            .next_back()
            .ok_or(MmioError::UnmappedAccess(addr))?;

        // Verify address is actually within this region
        if addr >= region.base_addr && addr < region.base_addr + region.size {
            Ok(region)
        } else {
            Err(MmioError::UnmappedAccess(addr))
        }
    }

    /// find a overlapping region if it exists, O(log n)
    fn find_overlap(&self, base: u64, size: u64) -> Option<(u64, u64)> {
        let new_end = base + size;

        if let Some((_, region)) = self.regions.range(base..).next() {
            if region.base_addr < new_end {
                return Some((region.base_addr, region.base_addr + region.size));
            }
        }

        if let Some((_, region)) = self.regions.range(..base).next_back() {
            let existing_end = region.base_addr + region.size;
            if existing_end > base {
                return Some((region.base_addr, existing_end));
            }
        }

        None
    }
}
