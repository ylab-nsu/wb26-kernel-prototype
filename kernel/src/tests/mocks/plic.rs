use alloc::vec::Vec;
use crate::arch::traits::TargetInterruptController;

pub struct MockInterruptController;

static mut ENABLED_IRQS: Vec<u32> = Vec::new();

impl MockInterruptController {
    pub fn reset() {
        unsafe {
            ENABLED_IRQS.clear();
        }
    }

    pub fn get_enabled_irqs() -> Vec<u32> {
        unsafe { ENABLED_IRQS.clone() }
    }
}

impl TargetInterruptController for MockInterruptController {
    fn enable_irq(irq: u32) {
        unsafe {
            ENABLED_IRQS.push(irq);
        }
    }
}
