use crate::drivers::TEST_DRIVER_ANY;
use crate::mmu::set_satp;
use crate::threading::thread::{
    get_current_thread_id, get_process, get_process_count, set_current_thread_id, Context,
    PROCESSES_INDEXES,
};

extern "C" {
    fn _switch_thread(curr_context: &mut Context, next_context: &mut Context) -> usize;
}

static mut LAST_USER: usize = unsafe { PROCESSES_INDEXES.user_start - 1 };

#[export_name = "_reschedule_rust"]
pub(crate) fn reschedule() {
    let time = riscv::register::time::read64();
    sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");
    // sbi::timer::set_timer(time + 1_000).expect("Can't set timer");

    let next_thread = unsafe {
        if TEST_DRIVER_ANY {
            PROCESSES_INDEXES.driver_task
        } else {
            let mut checked = LAST_USER;
            let next_thread = loop {
                checked = if checked < get_process_count() - 1 {
                    checked + 1
                } else {
                    PROCESSES_INDEXES.user_start
                };
                if get_process(checked).valid {
                    break checked;
                }
            };
            LAST_USER = next_thread;
            next_thread
        }
    };

    let (curr, next) = unsafe {
        let curr = get_process(get_current_thread_id());
        let next = get_process(next_thread);

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

    println!("Switching from {} to {}", curr.id, next.id);
    unsafe { _switch_thread(curr.context, next.context) };
}
