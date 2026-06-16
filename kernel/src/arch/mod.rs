use static_assertions::assert_impl_all;

use crate::arch::traits::{TargetAddress, TargetAddressSpace, TargetDebugWriter, TargetPlatform};

pub mod traits;

mod common;
mod macros;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "riscv64")] {
        mod riscv;
        pub type VirtualAddress = riscv::mmu::Sv39VirtualAddress;
        pub type PhysicalAddress = riscv::mmu::Sv39PhysicalAddress;
        pub type Platform = riscv::platform::RiscvPlatform;
        pub type DebugWriter = riscv::write::SbiWriter;
        pub type AddressSpace = riscv::vm::RiscvSv39AddressSpace;
    } else {
        compile_error!("Unsupported platform");
    }
}

assert_impl_all!(VirtualAddress: TargetAddress);
assert_impl_all!(PhysicalAddress: TargetAddress);
assert_impl_all!(Platform: TargetPlatform);
assert_impl_all!(DebugWriter: TargetDebugWriter);
assert_impl_all!(AddressSpace: TargetAddressSpace);
