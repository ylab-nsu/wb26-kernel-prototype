use crate::arch::traits::TargetContext;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct RiscvContext {
    pub ra: usize,
    pub sp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

impl TargetContext for RiscvContext {
    fn with_ra(mut self, ra: usize) -> Self {
        self.ra = ra;
        self
    }

    fn with_sp(mut self, sp: usize) -> Self {
        self.sp = sp;
        self
    }
}
