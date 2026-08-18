use static_assertions::assert_impl_all;

use crate::arch::traits::{TargetAddress, TargetAddressSpace, TargetContext, TargetDebugWriter, TargetMapping, TargetPhysicalAllocation, TargetPhysicalAllocator, TargetPlatform, TargetTrapFrame};

pub use crate::arch::riscv::threading::event::{TickInstant, add_timer};

pub mod traits;

mod common;
mod macros;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "riscv64")] {
        mod riscv;
        pub type VirtualAddress = riscv::vm::Sv39VirtualAddress;
        pub type PhysicalAddress = riscv::vm::Sv39PhysicalAddress;
        pub type PhysicalAllocator = riscv::alloc::phys::RiscvPhysicalAllocator;
        pub type PhysicalAllocation = riscv::alloc::RiscvPhysicalAllocation;
        pub type Platform = riscv::platform::RiscvPlatform;
        pub type DebugWriter = riscv::write::SbiWriter;
        pub type AddressSpace = riscv::vm::address_space::Sv39AddressSpace;
        pub type Mapping = riscv::vm::address_space::Sv39Mapping;
        
        pub type TrapFrame = riscv::threading::trap::RiscvTrapFrame;
        pub type Context = riscv::threading::switch::RiscvContext;

        pub use riscv::mmu::set_satp;
    } else {
        compile_error!("Unsupported platform");
    }
}

assert_impl_all!(VirtualAddress: TargetAddress);
assert_impl_all!(PhysicalAddress: TargetAddress);
assert_impl_all!(PhysicalAllocator: TargetPhysicalAllocator);
assert_impl_all!(PhysicalAllocation: TargetPhysicalAllocation);
assert_impl_all!(Platform: TargetPlatform);
assert_impl_all!(DebugWriter: TargetDebugWriter);
assert_impl_all!(AddressSpace: TargetAddressSpace);
assert_impl_all!(Mapping: TargetMapping);

assert_impl_all!(TrapFrame: TargetTrapFrame);
assert_impl_all!(Context: TargetContext);
