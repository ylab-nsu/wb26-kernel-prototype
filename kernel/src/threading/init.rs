use crate::arch::traits::TargetAddressSpace;
use crate::arch::traits::TargetPhysicalAllocator;
use crate::arch::{
    map_kernel_sections, AddressSpace, PhysicalAddress, PhysicalAllocator, VirtualAddress,
    KERNEL_LAYOUT,
};
use crate::drivers::driver_task;
use crate::threading::thread::{create_invalid_thread, spawn_kernel, spawn_user, THREADS_INDEXES};
use crate::vm::{MappingFlags, MappingPermissions};
use alloc::sync::Arc;
use core::arch::global_asm;

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
pub fn spawn_user_program_hardcoded_stack(prog: UserProgram) {
    let stack_end;
    unsafe {
        stack_end = NEXT_USER_STACK + 64 * 1024;
        if stack_end > MAX_USER_STACK {
            panic!("Stack area is exceeded");
        }
        NEXT_USER_STACK = stack_end;
    }
    let _ = spawn_user(unsafe { crt0 }, stack_end, prog.address_space, prog.entry);
}

pub fn setup_threads(kernel_address_space: Arc<AddressSpace>) {
    let time = riscv::register::time::read64();
    info!("Current time: {}", time);
    println!();
    println!();

    create_invalid_thread();

    let driver_task_id = spawn_kernel(driver_task, kernel_address_space);
    unsafe {
        THREADS_INDEXES.driver_task = driver_task_id;
        THREADS_INDEXES.user_start = driver_task_id + 1;
    }

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
        spawn_user_program_hardcoded_stack(prog);
    }
}
