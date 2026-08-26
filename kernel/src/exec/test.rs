//! Ad-hoc test harness: load a RAM-resident ELF and run it.
//!
//! `ELF_BASE` is where QEMU places the file (`-device loader,addr=...`).
use crate::exec::elf::run_elf;
use crate::exec::image::Image;

/// Physical address where QEMU loads the test ELF.
pub const ELF_BASE: usize = 0x8A00_0000;

/// Load `print.elf` and run it as a user thread in its own address space.
pub fn test_run_elf() {
    info!("exec test: loading ELF at {ELF_BASE:#x}");

    let mut image = Image::new(ELF_BASE as *const u8);
    let id = run_elf(&mut image).expect("run_elf");

    info!("exec test: spawned thread {id}");
}
