use std::arch::asm;

pub fn get_cntpct_el0() -> u64 {
    let physical_count: u64;

    // SAFETY: This assembly code reads the current physical counter value from the CNTVCT_EL0 register.
    unsafe {
        asm!("mrs {}, cntpct_el0", out(reg) physical_count);
    }

    physical_count
}
