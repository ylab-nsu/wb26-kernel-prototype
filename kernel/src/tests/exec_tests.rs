//! In-kernel tests for the `exec` module (run under the `kernel-unit-tests`
//! feature, executed at boot in QEMU).
//!
//! Pure ELF-validation logic is tested against crafted ELF bytes in a local
//! buffer. Mapping paths (`map_segment`, `init_stack`, `load`) run against the
//! real physical allocator (initialized by the normal boot) and a
//! [`MockAddressSpace`] that records every mapping call.

use allocator::AllocatorError;

use object::read::elf::FileHeader;
use object::{elf, LittleEndian};

use crate::allocator;
use crate::exec::elf::{
    analyze_phdrs, init_stack, load, load_phdrs, map_segment, prot_from, read_and_validate,
    validate_ehdr, validate_phdr, vaddr_alloc_bounds, Elf64,
};
use crate::exec::image::Image;
use crate::exec::{align_down, align_up, ExecError, USER_STACK_SIZE, USER_STACK_TOP};
use crate::tests::mocks::address_space::MockAddressSpace;
use crate::run_test;

const EHDR_SIZE: usize = 64;
const PHDR_SIZE: usize = 56;
const EM_RISCV: u16 = 243;

/// A `PT_LOAD` segment description for [`build_elf`].
struct Segment {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_filesz: u64,
    p_memsz: u64,
}

fn u16_at(buf: &mut [u8], off: usize, v: u16) {
    buf[off..off + 2].copy_from_slice(&v.to_le_bytes());
}

fn u32_at(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn u64_at(buf: &mut [u8], off: usize, v: u64) {
    buf[off..off + 8].copy_from_slice(&v.to_le_bytes());
}

/// Build a minimal valid ELF64 (ehdr + phdrs) into a buffer.
fn build_elf(segments: &[Segment]) -> [u8; 256] {
    let mut buf = [0u8; 256];
    buf[0] = 0x7f;
    buf[1] = b'E';
    buf[2] = b'L';
    buf[3] = b'F';
    buf[4] = 2; // ELFCLASS64
    buf[5] = 1; // ELFDATA2LSB
    buf[6] = 1; // EV_CURRENT
    u16_at(&mut buf, 16, 2); // e_type = ET_EXEC
    u16_at(&mut buf, 18, EM_RISCV);
    u32_at(&mut buf, 20, 1); // e_version
    u64_at(&mut buf, 24, 0x10000); // e_entry
    u64_at(&mut buf, 32, EHDR_SIZE as u64); // e_phoff
    u16_at(&mut buf, 52, EHDR_SIZE as u16); // e_ehsize
    u16_at(&mut buf, 54, PHDR_SIZE as u16); // e_phentsize
    u16_at(&mut buf, 56, segments.len() as u16); // e_phnum

    for (i, s) in segments.iter().enumerate() {
        let off = EHDR_SIZE + i * PHDR_SIZE;
        u32_at(&mut buf, off, s.p_type);
        u32_at(&mut buf, off + 4, s.p_flags);
        u64_at(&mut buf, off + 8, s.p_offset);
        u64_at(&mut buf, off + 16, s.p_vaddr);
        u64_at(&mut buf, off + 32, s.p_filesz);
        u64_at(&mut buf, off + 40, s.p_memsz);
    }
    buf
}

fn load_ehdr_from(buf: &[u8; 256]) -> &'static Elf64 {
    let image = Image::new(buf.as_ptr());
    Elf64::parse(image).unwrap()
}

fn load_phdrs_from(buf: &[u8; 256]) -> (&'static Elf64, &'static [crate::exec::elf::Phdr]) {
    let mut image = Image::new(buf.as_ptr());
    let ehdr = Elf64::parse(image).unwrap();
    let phdrs = load_phdrs(&mut image, ehdr, LittleEndian).unwrap();
    (ehdr, phdrs)
}

fn valid_segment() -> Segment {
    Segment {
        p_type: elf::PT_LOAD.0,
        p_flags: 6, // PF_R | PF_W
        p_offset: 0,
        p_vaddr: 0x1000,
        p_filesz: 0x20,
        p_memsz: 0x20,
    }
}

fn test_align_down_up() {
    assert_eq!(align_down(0x1234, 0x1000), 0x1000);
    assert_eq!(align_down(0x1000, 0x1000), 0x1000);
    assert_eq!(align_up(0x1234, 0x1000), 0x2000);
    assert_eq!(align_up(0x1000, 0x1000), 0x1000);
}

fn test_exec_error_from() {
    assert_eq!(
        ExecError::from(AllocatorError::NotEnoughMemory),
        ExecError::OutOfMemory
    );
    assert_eq!(
        ExecError::from(AllocatorError::AlreadyOccupied),
        ExecError::OutOfMemory
    );
    assert_eq!(
        ExecError::from(AllocatorError::InvalidAddress),
        ExecError::InvalidAddress
    );
}

fn test_validate_ehdr_valid() {
    let buf = build_elf(&[valid_segment()]);
    let ehdr = load_ehdr_from(&buf);
    assert_eq!(validate_ehdr(ehdr, LittleEndian), Ok(()));
}

fn test_validate_ehdr_bad_magic() {
    let mut buf = build_elf(&[valid_segment()]);
    buf[0] = 0;
    let mut image = Image::new(buf.as_ptr());
    assert!(matches!(read_and_validate(&mut image), Err(ExecError::BadMagic)));
}

fn test_validate_ehdr_class32() {
    let mut buf = build_elf(&[valid_segment()]);
    buf[4] = 1; // ELFCLASS32
    // `Elf64::parse` itself rejects a non-64-bit class, so the failure
    // surfaces as `BadMagic` from `read_and_validate`.
    let mut image = Image::new(buf.as_ptr());
    assert!(matches!(
        read_and_validate(&mut image),
        Err(ExecError::BadMagic)
    ));
}

fn test_validate_ehdr_big_endian() {
    let mut buf = build_elf(&[valid_segment()]);
    buf[5] = 2; // ELFDATA2MSB
    let ehdr = load_ehdr_from(&buf);
    assert_eq!(validate_ehdr(ehdr, LittleEndian), Err(ExecError::UnsupportedEndian));
}

fn test_validate_ehdr_wrong_machine() {
    let mut buf = build_elf(&[valid_segment()]);
    u16_at(&mut buf, 18, 62); // EM_X86_64
    let ehdr = load_ehdr_from(&buf);
    assert_eq!(validate_ehdr(ehdr, LittleEndian), Err(ExecError::UnsupportedMachine));
}

fn test_validate_ehdr_dynamic() {
    let mut buf = build_elf(&[valid_segment()]);
    u16_at(&mut buf, 16, 3); // ET_DYN
    let ehdr = load_ehdr_from(&buf);
    assert_eq!(validate_ehdr(ehdr, LittleEndian), Err(ExecError::DynamicUnsupported));
}

fn test_validate_ehdr_wrong_type() {
    let mut buf = build_elf(&[valid_segment()]);
    u16_at(&mut buf, 16, 1); // ET_REL
    let ehdr = load_ehdr_from(&buf);
    assert_eq!(validate_ehdr(ehdr, LittleEndian), Err(ExecError::UnsupportedType));
}

fn test_validate_ehdr_bad_phentsize() {
    let mut buf = build_elf(&[valid_segment()]);
    u16_at(&mut buf, 54, 32); // wrong e_phentsize
    let ehdr = load_ehdr_from(&buf);
    assert_eq!(validate_ehdr(ehdr, LittleEndian), Err(ExecError::BadPhdrEntSize));
}

fn test_validate_ehdr_bad_phnum() {
    let buf = build_elf(&[valid_segment()]);
    let mut buf = buf;
    u16_at(&mut buf, 56, 0); // e_phnum = 0
    let ehdr = load_ehdr_from(&buf);
    assert_eq!(validate_ehdr(ehdr, LittleEndian), Err(ExecError::BadPhdrCount));
}

fn test_validate_phdr_memsz_lt_filesz() {
    let buf = build_elf(&[Segment { p_memsz: 0x10, p_filesz: 0x20, ..valid_segment() }]);
    let (_, phdrs) = load_phdrs_from(&buf);
    assert_eq!(
        validate_phdr(&phdrs[0], LittleEndian),
        Err(ExecError::MemszLtFilesz)
    );
}

fn test_validate_phdr_misaligned() {
    let buf = build_elf(&[Segment {
        p_offset: 0x10,
        ..valid_segment()
    }]);
    let (_, phdrs) = load_phdrs_from(&buf);
    assert_eq!(
        validate_phdr(&phdrs[0], LittleEndian),
        Err(ExecError::MisalignedSegment)
    );
}

fn test_vaddr_alloc_bounds() {
    let buf = build_elf(&[Segment {
        p_vaddr: 0x1234,
        p_memsz: 0x20,
        ..valid_segment()
    }]);
    let (_, phdrs) = load_phdrs_from(&buf);
    let (start, end) = vaddr_alloc_bounds(&phdrs[0], LittleEndian);
    assert_eq!(start, 0x1000);
    assert_eq!(end, 0x2000);
}

fn test_prot_from() {
    let r = prot_from(4); // PF_R
    assert!(r.read());
    assert!(!r.write());
    assert!(!r.execute());

    let w = prot_from(2); // PF_W
    assert!(!w.read());
    assert!(w.write());

    let x = prot_from(1); // PF_X
    assert!(x.execute());

    let rwx = prot_from(7);
    assert!(rwx.read());
    assert!(rwx.write());
    assert!(rwx.execute());
}

fn test_analyze_phdrs_span() {
    let buf = build_elf(&[
        Segment {
            p_vaddr: 0x1000,
            p_memsz: 0x100,
            ..valid_segment()
        },
        Segment {
            p_vaddr: 0x2000,
            p_memsz: 0x200,
            ..valid_segment()
        },
    ]);
    let (_, phdrs) = load_phdrs_from(&buf);
    let analysis = analyze_phdrs(phdrs, LittleEndian).unwrap();
    assert_eq!(analysis.min_vaddr, 0x1000);
    assert_eq!(analysis.max_vaddr, 0x3000);
    assert_eq!(analysis.load_count, 2);
}

fn test_analyze_phdrs_interp() {
    let buf = build_elf(&[
        valid_segment(),
        Segment {
            p_type: elf::PT_INTERP.0,
            ..valid_segment()
        },
    ]);
    let (_, phdrs) = load_phdrs_from(&buf);
    assert!(matches!(
        analyze_phdrs(phdrs, LittleEndian),
        Err(ExecError::InterpreterUnsupported)
    ));
}

fn test_analyze_phdrs_no_load() {
    let buf = build_elf(&[Segment {
        p_type: 0, // PT_NULL
        ..valid_segment()
    }]);
    let (_, phdrs) = load_phdrs_from(&buf);
    assert!(matches!(
        analyze_phdrs(phdrs, LittleEndian),
        Err(ExecError::NoLoadSegments)
    ));
}

fn test_map_segment() {
    let buf = build_elf(&[valid_segment()]);
    let mut image = Image::new(buf.as_ptr());
    let (_, phdrs) = load_phdrs_from(&buf);

    let mut address_space = MockAddressSpace::new();
    map_segment(&mut image, &mut address_space, &phdrs[0], LittleEndian).unwrap();

    assert_eq!(address_space.records.len(), 1);
    let rec = &address_space.records[0];
    assert_eq!(rec.vaddr, 0x1000);
    assert_eq!(rec.size, 0x1000);
    assert!(rec.perms.read());
    assert!(rec.perms.write());
    assert!(!rec.perms.execute());
    assert!(rec.flags.user());

    // The file bytes (filesz 0x20 at p_offset 0) were copied into the mapped
    // region; segment_offset is 0 for a page-aligned segment.
    let dst = unsafe { core::slice::from_raw_parts(rec.phys_addr as *const u8, 0x20) };
    assert_eq!(dst, &buf[0..0x20]);
}

fn test_init_stack() {
    let mut address_space = MockAddressSpace::new();
    init_stack(&mut address_space);

    assert_eq!(address_space.records.len(), 1);
    let rec = &address_space.records[0];
    assert_eq!(rec.vaddr, (USER_STACK_TOP - USER_STACK_SIZE) as usize);
    assert_eq!(rec.size, USER_STACK_SIZE as usize);
    assert!(rec.perms.write());
    assert!(rec.flags.user());
}

fn test_load() {
    let buf = build_elf(&[
        valid_segment(),
        Segment {
            p_vaddr: 0x2000,
            p_memsz: 0x100,
            ..valid_segment()
        },
    ]);
    let mut image = Image::new(buf.as_ptr());
    let meta = read_and_validate(&mut image).unwrap();

    let address_space = MockAddressSpace::new();
    let program = load(&meta, &mut image, address_space).unwrap();

    assert_eq!(program.entry, 0x10000);
    assert_eq!(program.sp, USER_STACK_TOP as usize);
    // stack + two segments
    assert_eq!(program.address_space.records.len(), 3);
    assert_eq!(program.address_space.records[0].vaddr, (USER_STACK_TOP - USER_STACK_SIZE) as usize);
    assert_eq!(program.address_space.records[1].vaddr, 0x1000);
    assert_eq!(program.address_space.records[2].vaddr, 0x2000);
}

pub fn run_tests() {
    run_test!(test_align_down_up);
    run_test!(test_exec_error_from);
    run_test!(test_validate_ehdr_valid);
    run_test!(test_validate_ehdr_bad_magic);
    run_test!(test_validate_ehdr_class32);
    run_test!(test_validate_ehdr_big_endian);
    run_test!(test_validate_ehdr_wrong_machine);
    run_test!(test_validate_ehdr_dynamic);
    run_test!(test_validate_ehdr_wrong_type);
    run_test!(test_validate_ehdr_bad_phentsize);
    run_test!(test_validate_ehdr_bad_phnum);
    run_test!(test_validate_phdr_memsz_lt_filesz);
    run_test!(test_validate_phdr_misaligned);
    run_test!(test_vaddr_alloc_bounds);
    run_test!(test_prot_from);
    run_test!(test_analyze_phdrs_span);
    run_test!(test_analyze_phdrs_interp);
    run_test!(test_analyze_phdrs_no_load);
    run_test!(test_map_segment);
    run_test!(test_init_stack);
    run_test!(test_load);
}