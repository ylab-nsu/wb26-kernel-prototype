use alloc::vec;
use alloc::vec::Vec;
use bitmap_allocator::BitAlloc64K;
use fdt::Fdt;

use crate::{
    allocator::{bitmap::BitmapMemoryAllocator, AllocatorError},
    arch::{
        riscv::alloc::RiscvPhysicalAllocation, riscv::vm::PAGE_SIZE,
        traits::{TargetAddress, TargetPhysicalAllocator},
        PhysicalAddress, PhysicalAllocation,
    },
    sync::{Mutex, Once},
    vm::{MappingFlags, MappingPermissions},
};

type RiscvPhysicalAllocatorInner = BitmapMemoryAllocator<PhysicalAddress, BitAlloc64K, PAGE_SIZE>;

/// A mapping attached to a physical page. Lives and dies with the page: the
/// page owns its mappings (reverse mapping, `AS -> page -> mapping`). It does
/// NOT reference the address space — only the physical page and the virtual
/// position/perms where it is mapped. PTE teardown on death is handled by the
/// refcount cascade (page dies only when the last owning AS is gone, so its
/// tables die too).
#[derive(Clone, Copy, Debug)]
pub struct PageMapping {
    pub phys_addr: usize,
    pub vaddr: usize,
    pub permissions: MappingPermissions,
    pub flags: MappingFlags,
}

struct PhysicalMemoryState {
    base: PhysicalAddress,
    allocator: RiscvPhysicalAllocatorInner,
    /// Per-page reference count (index = page offset from `base` / PAGE_SIZE).
    /// A page is returned to the allocator when its count drops to zero.
    refs: Vec<u8>,
    /// Mappings attached to pages (one entry per mapped page). Each entry
    /// carries its own `phys_addr`; mappings are removed when their page is
    /// deallocated (dies with the page). Indexed by page to keep the reverse
    /// mapping, but stored flat so the footprint scales with real mappings,
    /// not with total RAM.
    mappings: Vec<PageMapping>,
}

static PHYSICAL_MEMORY: Once<Mutex<PhysicalMemoryState>> = Once::new();

pub struct RiscvPhysicalAllocator;

impl RiscvPhysicalAllocator {
    pub(in crate::arch::riscv) unsafe fn init(base: PhysicalAddress, size: usize) {
        PHYSICAL_MEMORY.call_once(|| unsafe {
            let refs = vec![0u8; size / PAGE_SIZE];
            Mutex::new(PhysicalMemoryState {
                base,
                allocator: BitmapMemoryAllocator::new(base, size),
                refs,
                mappings: Vec::new(),
            })
        });
    }

    fn get_state() -> &'static Mutex<PhysicalMemoryState> {
        unsafe { PHYSICAL_MEMORY.get_unchecked() }
    }

    fn page_index(state: &PhysicalMemoryState, addr: PhysicalAddress) -> usize {
        addr.byte_offset_from_unsigned(state.base) / PAGE_SIZE
    }

    /// Take one more reference on already-allocated pages (no bitmap change).
    /// Used to share the same physical pages into another address space.
    pub fn retain(
        addr: PhysicalAddress,
        size: usize,
    ) -> Result<PhysicalAllocation, AllocatorError> {
        let mut state = Self::get_state().lock();
        let size = size.next_multiple_of(PAGE_SIZE);

        for off in (0..size).step_by(PAGE_SIZE) {
            let idx = Self::page_index(&state, addr.byte_add(off));
            let r = state.refs.get_mut(idx).ok_or(AllocatorError::InvalidAddress)?;
            if *r == 0 {
                return Err(AllocatorError::AlreadyOccupied);
            }
            *r += 1;
        }

        Ok(RiscvPhysicalAllocation::new(addr, size))
    }

    fn mark_owned(state: &mut PhysicalMemoryState, addr: PhysicalAddress, size: usize) {
        for off in (0..size).step_by(PAGE_SIZE) {
            let idx = Self::page_index(state, addr.byte_add(off));
            state.refs[idx] = 1;
        }
    }

    pub(in crate::arch::riscv) unsafe fn dealloc_contiguous(addr: PhysicalAddress, size: usize) {
        let mut state = Self::get_state().lock();

        for off in (0..size).step_by(PAGE_SIZE) {
            let idx = Self::page_index(&state, addr.byte_add(off));

            let do_dealloc = {
                let r = state.refs.get_mut(idx).expect("dealloc out of range");
                debug_assert!(*r > 0, "double free of physical page");
                let last = *r == 1;
                *r -= 1;
                last
            };

            if do_dealloc {
                // The page dies: drop its attached mappings, then return the
                // page to the allocator.
                let pa_usize = addr.byte_add(off).into_bits() as usize;
                state.mappings.retain(|m| m.phys_addr != pa_usize);
                unsafe {
                    state.allocator.dealloc_contiguous(addr.byte_add(off), PAGE_SIZE);
                }
            }
        }
    }

    fn mark_invalid(addr: PhysicalAddress, size: usize) {
        let mut state = Self::get_state().lock();
        state.allocator.alloc_contiguous_at(addr, size).unwrap();
        Self::mark_owned(&mut state, addr, size);
    }
}

/// Attach a mapping to every page of `addr..addr+size`.
///
/// Mappings are owned by the pages (reverse mapping): the page carries the
/// virtual positions/perms where it is mapped. Multiple address spaces can map
/// the same page (via `retain`), and all their mappings accumulate here. When
/// the page dies (last reference), its mappings die with it.
pub fn add_mapping(addr: PhysicalAddress, size: usize, mapping: PageMapping) {
    let mut state = RiscvPhysicalAllocator::get_state().lock();
    for off in (0..size).step_by(PAGE_SIZE) {
        let pa = addr.byte_add(off);
        if !state
            .mappings
            .iter()
            .any(|m| m.phys_addr == pa.into_bits() as usize && m.vaddr == mapping.vaddr)
        {
            state.mappings.push(PageMapping {
                phys_addr: pa.into_bits() as usize,
                vaddr: mapping.vaddr,
                permissions: mapping.permissions,
                flags: mapping.flags,
            });
        }
    }
}

/// Read the mappings attached to a page.
pub fn mappings_of(addr: PhysicalAddress) -> Vec<PageMapping> {
    let state = RiscvPhysicalAllocator::get_state().lock();
    state
        .mappings
        .iter()
        .copied()
        .filter(|m| m.phys_addr == addr.into_bits() as usize)
        .collect()
}

/// Detach a mapping (by virtual address) from every page of a range.
pub fn remove_mapping(addr: PhysicalAddress, size: usize, vaddr: usize) {
    let mut state = RiscvPhysicalAllocator::get_state().lock();
    for off in (0..size).step_by(PAGE_SIZE) {
        let pa = addr.byte_add(off);
        let pa_usize = pa.into_bits() as usize;
        state
            .mappings
            .retain(|m| !(m.phys_addr == pa_usize && m.vaddr == vaddr));
    }
}

impl TargetPhysicalAllocator for RiscvPhysicalAllocator {
    fn alloc_contiguous(size: usize) -> Result<PhysicalAllocation, AllocatorError> {
        let mut state = Self::get_state().lock();
        let (addr, size) = state.allocator.alloc_contiguous(size)?;
        Self::mark_owned(&mut state, addr, size);

        Ok(RiscvPhysicalAllocation::new(addr, size))
    }

    fn alloc_contiguous_aligned(
        size: usize,
        alignment: usize,
    ) -> Result<PhysicalAllocation, AllocatorError> {
        let mut state = Self::get_state().lock();
        let (addr, size) = state.allocator.alloc_contiguous_aligned(size, alignment)?;
        Self::mark_owned(&mut state, addr, size);

        Ok(RiscvPhysicalAllocation::new(addr, size))
    }

    fn alloc_contiguous_at(
        addr: PhysicalAddress,
        size: usize,
    ) -> Result<PhysicalAllocation, AllocatorError> {
        let mut state = Self::get_state().lock();
        let (addr, size) = state.allocator.alloc_contiguous_at(addr, size)?;
        Self::mark_owned(&mut state, addr, size);

        Ok(RiscvPhysicalAllocation::new(addr, size))
    }
}

pub fn init_physical_allocator(fdt: &Fdt) {
    let memory = fdt
        .memory()
        .regions()
        .next()
        .expect("Memory regions are not defined");

    let base = PhysicalAddress::try_from(memory.starting_address as usize).unwrap();
    let size = memory.size.unwrap();

    unsafe {
        RiscvPhysicalAllocator::init(base, size);
    };
    info!("Initialized physical memory allocator at address {base:p} with size 0x{size:x}");

    if fdt.memory_reservations().count() != 0 {
        panic!("Memory reservasion blocks in DTB are not supported!");
    }

    if let Some(reserved_regions) = fdt.find_node("/reserved-memory") {
        for child in reserved_regions.children() {
            let region = child.reg().unwrap().next().unwrap();

            let addr = PhysicalAddress::try_from(region.starting_address as usize).unwrap();
            let size = region.size.unwrap();

            RiscvPhysicalAllocator::mark_invalid(addr, size);
            info!("Reserved memory at {addr:p} with size 0x{size:x}");
        }
    } else {
        info!("No reserved memory regions found");
    }
}