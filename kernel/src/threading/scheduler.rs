use crate::mmu::set_satp;
use crate::threading::thread::{
    get_current_thread_id, get_process, get_process_count, set_current_thread_id, TrapFrame,
};

#[export_name = "_reschedule_rust"]
fn reschedule(/*frame: &mut TrapFrame*/) -> *mut TrapFrame {
    let time = riscv::register::time::read64();
    sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");
    // sbi::timer::set_timer(time + 1_000).expect("Can't set timer");

    let next_thread = unsafe {
        if get_current_thread_id() < get_process_count() - 1 {
            get_current_thread_id() + 1
        } else {
            1
        }
    };

    let (_curr, next) = unsafe {
        let curr = get_process(get_current_thread_id());
        let next = get_process(next_thread);

        // println!("CURRENT_THREAD: {CURRENT_THREAD}");

        (curr, next)
    };

    // if frame.pc == 0 {
    //     let epc = unsafe { riscv::register::sepc::read() };
    //     println!("frame.pc is zero, epc={:x}", epc);
    // }

    unsafe {
        set_current_thread_id(next_thread);
        riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::User);
        set_satp(1);
    };

    next.frame
}
