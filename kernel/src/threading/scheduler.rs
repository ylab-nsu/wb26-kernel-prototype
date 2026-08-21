use crate::arch::set_satp;
use crate::arch::traits::TargetContext;
use crate::drivers::TEST_DRIVER_ANY;
use crate::threading::thread::{
    get_current_thread_id, get_thread, get_threads_count, set_current_thread_id, THREADS_INDEXES,
};

unsafe fn switch_thread(
    curr_context: &mut impl TargetContext,
    next_context: &mut impl TargetContext,
) {
    extern "C" {
        fn _switch_thread(curr_context: usize, next_context: usize) -> usize;
    }
    unsafe {
        _switch_thread(
            curr_context as *mut _ as usize,
            next_context as *mut _ as usize,
        )
    };
}

static mut LAST_USER: usize = unsafe { THREADS_INDEXES.user_start - 1 };

#[export_name = "_reschedule_rust"]
pub fn reschedule() {
    // let time = riscv::register::time::read64();
    // sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");
    info!("Reschedule");
    // sbi::timer::set_timer(time + 1_000).expect("Can't set timer");

    let next_thread = unsafe {
        if TEST_DRIVER_ANY {
            THREADS_INDEXES.driver_task
        } else {
            let mut checked = LAST_USER;
            let next_thread = loop {
                checked = if checked < get_threads_count() - 1 {
                    checked + 1
                } else {
                    THREADS_INDEXES.user_start
                };
                if get_thread(checked).valid {
                    break checked;
                }
            };
            LAST_USER = next_thread;
            next_thread
        }
    };

    let (curr, next) = unsafe {
        let curr = get_thread(get_current_thread_id());
        let next = get_thread(next_thread);

        // println!("CURRENT_THREAD: {CURRENT_THREAD}");

        (curr, next)
    };

    unsafe {
        set_current_thread_id(next_thread);
        if next.is_kernel {
            riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::Supervisor);
            set_satp(0);
        } else {
            // Setting SPP here will work only without nested interrupts
            riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::User);
            set_satp(1);
        }
    };

    info!("Switching threads from {} to {}", curr.id, next.id);
    unsafe { switch_thread(curr.context, next.context) };
}
