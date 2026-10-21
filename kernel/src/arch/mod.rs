use static_assertions::assert_impl_all;

use crate::arch::traits::{
    TargetAddress, TargetAddressSpace, TargetAlarm, TargetContext, TargetDebugWriter,
    TargetMapping, TargetPhysicalAllocation, TargetPhysicalAllocator, TargetPlatform,
    TargetTimeConverter, TargetTimer, TargetTrapFrame,
};

pub mod traits;

mod common;
mod macros;

cfg_select! {
    target_arch = "riscv64" => {
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

        pub use riscv::mapping::map_kernel_sections;
        pub use riscv::memory::layout::KERNEL_LAYOUT;

        pub mod time {
            pub type PlatformTicks = super::riscv::time::Ticks;
            pub type PlatformTemporal = super::riscv::time::Temporal;
            pub type Timer = super::riscv::time::Timer;
            pub type TimeConverter = super::riscv::time::Timer;
            pub type Alarm = super::riscv::time::Alarm;
        }
    }
    _ => {
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

assert_impl_all!(time::PlatformTicks: num_traits::PrimInt, num_traits::Unsigned);
assert_impl_all!(time::PlatformTemporal: num_traits::PrimInt, num_traits::Unsigned);

assert_impl_all!(time::Timer: TargetTimer);
assert_impl_all!(time::TimeConverter: TargetTimeConverter);
assert_impl_all!(time::Alarm: TargetAlarm);
