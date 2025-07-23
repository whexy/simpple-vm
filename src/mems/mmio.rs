// MMIO

// For each device, it claims a region of memory.
// But the memory will never be mapped, or even exists in the actual RAM device.
//
// We use MMIO Registors, each assigned with a guest physical address
// When trapped, we check if any register is accessed.
// Each register should have a callback for being read and write.
// For reading: we decode the instruction, and finds out which register to set.

// To properly read the instruction, we should use a virtual memory management unit.
