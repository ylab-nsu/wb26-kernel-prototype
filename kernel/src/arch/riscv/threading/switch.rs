use crate::arch::traits::TargetContext;
use core::mem::MaybeUninit;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct RiscvContext(MaybeUninit<RiscvContextInner>);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RiscvContextInner {
    ra: usize,
    sp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

impl TargetContext for RiscvContext {
    unsafe fn new(initial_context_pc: usize, initial_context_sp: usize) -> Self {
        let mut context = RiscvContext(MaybeUninit::uninit());
        let p = context.0.as_mut_ptr();
        unsafe {
            (*p).ra = initial_context_pc;
            (*p).sp = initial_context_sp;
        }
        context
    }

    fn get_ptr(&mut self) -> *mut core::ffi::c_void {
        self.0.as_mut_ptr() as *mut core::ffi::c_void
    }
}
