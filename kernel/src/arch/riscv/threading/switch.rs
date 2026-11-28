use crate::arch::traits::TargetContext;
use core::mem::MaybeUninit;
use pinned_init::{init_from_closure, Init};

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
    fn init(pc: usize, sp: usize) -> impl Init<Self> {
        unsafe {
            init_from_closure(move |slot: *mut Self| {
                let p = (*slot).0.as_mut_ptr();
                (&raw mut (*p).ra).write(pc);
                (&raw mut (*p).sp).write(sp);

                Ok(())
            })
        }
    }

    const NEW_STUB: Self = Self(MaybeUninit::uninit());
}
