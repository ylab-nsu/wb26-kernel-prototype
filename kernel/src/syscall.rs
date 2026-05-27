use crate::drivers::TEST_DRIVER_QUEUE;
use crate::threading::scheduler::reschedule;
use riscv::_export::critical_section;

pub fn handle_syscall(
    syscall_number: usize,
    arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
) {
    match syscall_number {
        1 => loop {
            if critical_section::with(|_| {
                for _ in 0..2 {
                    match TEST_DRIVER_QUEUE.enqueue(arg1 as i32) {
                        Ok(()) => return true,
                        Err(_) => {}
                    }
                }
                println!("Cannot put element into queue, reschedule");
                reschedule();
                false
            }) {
                break;
            }
        },
        _ => println!("Unexpected syscall number: {}", syscall_number),
    }
}
