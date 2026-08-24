//! Ad-hoc test harness: load a RAM-resident ELF into a fresh address space
//! and spawn it as a user thread with its own page tables.
//!
//! `ELF_BASE` is where QEMU places the file (`-device loader,addr=...`).

use crate::arch::traits::{TargetAddressSpace, TargetPhysicalAllocator};
use crate::arch::{AddressSpace, PhysicalAllocator, VirtualAddress};
use crate::exec::elf::{elf_load, map_kernel_shared};
use crate::exec::image::Image;
use crate::vm::{MappingFlags, MappingPermissions};
use crate::threading::thread::spawn_user_in_shared;

/// Physical address where QEMU loads the test ELF.
pub const ELF_BASE: usize = 0x8A00_0000;
/// User stack placement for the spawned program.
pub const USER_STACK_TOP: u64 = 0x8C00_0000;
pub const USER_STACK_SIZE: u64 = 16 * 4096;

/// Load `print.elf` into a fresh address space and spawn user threads sharing
/// that same address space.
pub fn run_elf() {
    info!("exec test: loading ELF at {ELF_BASE:#x}");

    let mut image = Image::new(ELF_BASE as *const u8);
    let mut address_space = AddressSpace::new();

    map_kernel_shared(&mut address_space).expect("map kernel shared");

    let info = elf_load(&mut image, &mut address_space, 0).expect("elf_load");

    // Two stacks, one per thread, both inside the same (shared) address space.
    let stack_alloc =
        PhysicalAllocator::alloc_contiguous(USER_STACK_SIZE as usize).expect("alloc stack 1");
    address_space.map(
        VirtualAddress::try_from(USER_STACK_TOP as usize).unwrap(),
        stack_alloc,
        MappingPermissions::rw(),
        MappingFlags::new().with_user(true),
    );

    let stack2_top = USER_STACK_TOP + USER_STACK_SIZE;
    let stack_alloc2 =
        PhysicalAllocator::alloc_contiguous(USER_STACK_SIZE as usize).expect("alloc stack 2");
    address_space.map(
        VirtualAddress::try_from(stack2_top as usize).unwrap(),
        stack_alloc2,
        MappingPermissions::rw(),
        MappingFlags::new().with_user(true),
    );

    // Two threads share the same address space: cloning the AS takes one more
    // reference on its root page table and every page it holds. The AS and its
    // pages live until the last thread using them dies.
    let id1 = spawn_user_in_shared(info.entry as usize, USER_STACK_TOP as usize, &address_space);
    let id2 = spawn_user_in_shared(info.entry as usize, stack2_top as usize, &address_space);

    info!(
        "exec test: spawned threads {id1} and {id2} sharing one address space, entry={:#018x} brk={:#018x} stack_exec={}",
        info.entry, info.brk, info.stack_exec,
    );
}