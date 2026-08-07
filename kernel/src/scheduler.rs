use crate::arch::{set_satp, TrapFrame};

pub fn reschedule(frame: &mut TrapFrame) {
    let time = riscv::register::time::read64();
    sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");
    // sbi::timer::set_timer(time + 1_000).expect("Can't set timer");

    let next_thread = unsafe {
        if crate::thread::CURRENT_THREAD < crate::thread::PROCESSES.len() - 1 {
            crate::thread::CURRENT_THREAD + 1
        } else {
            1
        }
    };

    let (curr, next) = unsafe {
        let curr = crate::thread::PROCESSES
            .get_mut(crate::thread::CURRENT_THREAD)
            .unwrap();
        let next = crate::thread::PROCESSES.get_mut(next_thread).unwrap();

        // println!("CURRENT_THREAD: {CURRENT_THREAD}");

        (curr, next)
    };

    let t = frame.clone();
    curr.frame = t;
    *frame = next.frame.clone();

    unsafe {
        crate::thread::CURRENT_THREAD = next_thread;
        riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::User);
        set_satp(1);
    };
}
