use static_assertions::assert_impl_all;

use crate::arch::traits::{TargetAddressSpace, TargetPlatform};

pub mod traits;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "riscv64")] {
        mod riscv;
        pub type Platform = riscv::platform::RiscvPlatform;
        pub type AddressSpace = riscv::vm::RiscvSv39AddressSpace;
    } else {
        compile_error!("Unsupported platform");
    }
}

assert_impl_all!(Platform: TargetPlatform);
assert_impl_all!(AddressSpace: TargetAddressSpace);
