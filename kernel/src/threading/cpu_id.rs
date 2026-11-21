use core::num::NonZero;

use static_assertions::{assert_eq_align, assert_eq_size};

type RawCpuId = usize;

/// Identifier of a logical processor.
/// 
/// This type represents ordinal indentifier of a logical processor core,
/// also sometimes referred to as hardware thread.
/// 
/// This implementation uses NVO with XOR-trick so that niche of that type
/// is non-zero.
#[derive(Debug)]
#[repr(transparent)]
pub struct CpuId {
    inner: NonZero<RawCpuId>,
}

impl CpuId {
    const NICHE: RawCpuId = RawCpuId::MAX;

    // Can't make this function const in current stable Rust
    pub fn new(id: RawCpuId) -> Option<Self> {
        NonZero::new(id ^ Self::NICHE).and_then(|v| Some(Self { inner: v }))
    }

    pub const unsafe fn new_unchecked(id: RawCpuId) -> Self {
        Self {
            inner: unsafe { NonZero::new_unchecked(id ^ Self::NICHE) },
        }
    }

    pub const fn get(self) -> RawCpuId {
        self.inner.get() ^ Self::NICHE
    }
}

impl CpuId {
    pub const fn option_to_raw(this: Option<Self>) -> RawCpuId {
        unsafe { core::mem::transmute(this) }
    }

    pub const unsafe fn option_from_raw(raw: RawCpuId) -> Option<Self> {
        unsafe { core::mem::transmute(raw) }
    }
}

assert_eq_size!(CpuId, RawCpuId);
assert_eq_align!(CpuId, RawCpuId);

assert_eq_size!(CpuId, Option<CpuId>);
assert_eq_align!(CpuId, Option<CpuId>);
