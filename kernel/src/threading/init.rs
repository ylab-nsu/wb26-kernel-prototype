use crate::arch::traits::TargetPhysicalAllocator;
use crate::arch::traits::{TargetAddressSpace, TargetPlatform};
use crate::arch::{
    map_kernel_sections, AddressSpace, PhysicalAddress, PhysicalAllocator, Platform,
    VirtualAddress, KERNEL_LAYOUT,
};
use crate::drivers::driver_task;
use crate::executor::executor;
use crate::threading::scheduler::schedule_thread;
use crate::threading::thread::{
    pseudo_allocate_kernel_stack, ThreadBuilder, ThreadRef, THREADS_COUNT,
};
use crate::vm::{MappingFlags, MappingPermissions};
use alloc::sync::Arc;
use core::arch::global_asm;
use core::num::NonZeroUsize;
use core::sync::atomic::Ordering;

pub unsafe fn create_initial_invalid_thread() {
    let id = THREADS_COUNT.fetch_add(1, Ordering::AcqRel);
    let kernel_stack_bottom = pseudo_allocate_kernel_stack(id);
    let thread = ThreadBuilder::default()
        .id(id)
        .is_user(false)
        .not_valid(true)
        .initial_pc(1.try_into().unwrap())
        .kernel_stack(kernel_stack_bottom)
        .build();
    let thread = thread.try_claim().unwrap();
    let thread = thread.run();
    unsafe { Platform::set_current_thread_raw(&thread.into_raw()) }
}

pub fn spawn_user(
    f: extern "C" fn() -> !,
    user_sp: NonZeroUsize,
    address_space: Arc<AddressSpace>,
    user_main: usize,
) -> ThreadRef {
    let id = THREADS_COUNT.fetch_add(1, Ordering::AcqRel);
    let kernel_stack_bottom = pseudo_allocate_kernel_stack(id);
    let thread = ThreadBuilder::default()
        .id(id)
        .is_user(true)
        .address_space(address_space)
        .initial_pc(unsafe { NonZeroUsize::new_unchecked(f as usize) })
        .initial_user_sp(user_sp)
        .initial_arg0(user_main)
        .kernel_stack(kernel_stack_bottom)
        .build();
    thread
}

pub fn spawn_kernel(f: extern "C" fn() -> !, address_space: Arc<AddressSpace>) -> ThreadRef {
    let id = THREADS_COUNT.fetch_add(1, Ordering::AcqRel);
    let kernel_stack_bottom = pseudo_allocate_kernel_stack(id);
    let thread = ThreadBuilder::default()
        .id(id)
        .is_user(false)
        .address_space(address_space)
        .initial_pc(unsafe { NonZeroUsize::new_unchecked(f as usize) })
        .kernel_stack(kernel_stack_bottom)
        .build();
    thread
}

pub static mut NEXT_USER_STACK: usize = 0x47000000;
pub const MAX_USER_STACK: usize = 0x48000000;

pub struct UserProgram {
    pub entry: usize,
    pub sp: usize,
    pub address_space: Arc<AddressSpace>,
}

global_asm!(
    ".section .rodata.usersymaddrs, \"a\"",
    ".align 2",
    "crt0:     .dc.a __user_crt0",
    "user1:    .dc.a __user_user1",
    "og_process1: .dc.a __user_og_process1",
    "og_process2: .dc.a __user_og_process2",
    "og_process3: .dc.a __user_og_process3",
    "scull_user: .dc.a __user_scull_user",
);

extern "C" {
    static crt0: extern "C" fn() -> !;
    static user1: extern "C" fn();
    static og_process1: extern "C" fn();
    static og_process2: extern "C" fn();
    static og_process3: extern "C" fn();
    static scull_user: extern "C" fn();
}

#[inline]
pub fn spawn_user_program_hardcoded_stack(prog: UserProgram) -> ThreadRef {
    let stack_end;
    unsafe {
        stack_end = NEXT_USER_STACK + 64 * 1024;
        if stack_end > MAX_USER_STACK {
            panic!("Stack area is exceeded");
        }
        NEXT_USER_STACK = stack_end;
    }
    spawn_user(
        unsafe { crt0 },
        stack_end.try_into().unwrap(),
        prog.address_space,
        prog.entry,
    )
}

pub fn setup_threads(kernel_address_space: Arc<AddressSpace>) {
    let time = riscv::register::time::read64();
    info!("Current time: {}", time);
    println!();
    println!();

    unsafe { create_initial_invalid_thread() };

    let driver_task = spawn_kernel(driver_task, kernel_address_space.clone());
    schedule_thread(driver_task);

    let executor = spawn_kernel(executor, kernel_address_space.clone());
    info!("Spawned executor thread {}", executor.id);
    schedule_thread(executor);

    let user_as = Arc::new(AddressSpace::new());
    map_kernel_sections(&user_as);

    let user_start: u64 = 0x45000000;
    let user_phys =
        PhysicalAddress::try_from(user_start.wrapping_sub(KERNEL_LAYOUT.user_va_offset as u64))
            .unwrap();
    let user_alloc = PhysicalAllocator::alloc_contiguous_at(user_phys, 48 * 1024 * 1024).unwrap();

    user_as.map(
        VirtualAddress::from(user_start),
        Arc::new(user_alloc),
        MappingPermissions::rwx(),
        MappingFlags::new().with_user(true),
    );

    let user_programs: [UserProgram; _] = [
        UserProgram {
            entry: unsafe { user1 as usize },
            sp: 0,
            address_space: user_as.clone(),
        },
        UserProgram {
            entry: unsafe { og_process1 as usize },
            sp: 0,
            address_space: user_as.clone(),
        },
        UserProgram {
            entry: unsafe { og_process2 as usize },
            sp: 0,
            address_space: user_as.clone(),
        },
        UserProgram {
            entry: unsafe { og_process3 as usize },
            sp: 0,
            address_space: user_as.clone(),
        },
        UserProgram {
            entry: unsafe { scull_user as usize },
            sp: 0,
            address_space: user_as.clone(),
        },
    ];

    for prog in user_programs {
        let t = spawn_user_program_hardcoded_stack(prog);
        schedule_thread(t);
    }
}
