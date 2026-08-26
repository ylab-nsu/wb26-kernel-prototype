use crate::arch::AddressSpace;
use crate::drivers::driver_task;
use crate::threading::thread::{create_empty_thread, spawn_kernel, spawn_user_in, THREADS_INDEXES};

pub struct UserProgram {
    pub entry: usize,
    pub sp: usize,
    pub address_space: AddressSpace,
}

pub fn spawn_user_program(program: UserProgram) -> usize {
    spawn_user_in(program.entry, program.sp, program.address_space)
}

pub fn setup_threads() {
    let time = riscv::register::time::read64();
    info!("Current time: {}", time);
    println!();
    println!();

    create_empty_thread();

    let driver_task_id = spawn_kernel(driver_task);
    unsafe {
        THREADS_INDEXES.driver_task = driver_task_id;
        THREADS_INDEXES.user_start = driver_task_id + 1;
    }
}
