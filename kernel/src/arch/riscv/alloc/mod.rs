use crate::{
    arch::traits::TargetPhysicalAllocation,
    arch::{PhysicalAddress, PhysicalAllocator},
};

pub mod phys;

pub struct RiscvPhysicalAllocation {
    addr: PhysicalAddress,
    size: usize,
}

impl RiscvPhysicalAllocation {
    pub(crate) fn new(addr: PhysicalAddress, size: usize) -> Self {
        RiscvPhysicalAllocation { addr, size }
    }
}

impl core::fmt::Debug for RiscvPhysicalAllocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RiscvPhysicalAllocation")
            .field("addr", &format_args!("{:p}", self.addr))
            .field("size", &self.size)
            .finish()
    }
}

impl core::fmt::Display for RiscvPhysicalAllocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{} bytes at {:p}", self.size, self.addr))
    }
}

impl TargetPhysicalAllocation for RiscvPhysicalAllocation {
    fn addr(&self) -> PhysicalAddress {
        self.addr
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl Drop for RiscvPhysicalAllocation {
    fn drop(&mut self) {
        unsafe { PhysicalAllocator::dealloc_contiguous(self.addr, self.size) };
    }
}