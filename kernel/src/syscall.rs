use crate::drivers::{put_into_queue, TestDriverMessage, TEST_DRIVER_QUEUE};
use crate::threading::scheduler::reschedule;
use riscv::_export::critical_section;

pub fn handle_syscall(syscall_number: usize, arg1: usize, arg2: usize, _arg3: usize, _arg4: usize) {
    match syscall_number {
        1 => put_into_queue(
            TestDriverMessage::PrintNumber { number: arg1 },
            TEST_DRIVER_QUEUE.as_view(),
        ),
        2 => put_into_queue(
            TestDriverMessage::PrintString {
                user_addr: arg1,
                len: arg2,
            },
            TEST_DRIVER_QUEUE.as_view(),
        ),
        _ => println!("Unexpected syscall number: {}", syscall_number),
    }
}
