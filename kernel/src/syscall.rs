use crate::drivers::{put_into_queue, TestDriverMessage, TEST_DRIVER_QUEUE};
use crate::threading::scheduler::reschedule;

pub fn handle_syscall(syscall_number: usize, arg1: usize, arg2: usize, arg3: usize, _arg4: usize) {
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
        3 => put_into_queue(
            TestDriverMessage::WriteScull {
                user_src: arg1,
                scull_dst: arg2,
                count: arg3,
            },
            TEST_DRIVER_QUEUE.as_view(),
        ),
        4 => put_into_queue(
            TestDriverMessage::ReadScull {
                user_dst: arg1,
                scull_src: arg2,
                count: arg3,
            },
            TEST_DRIVER_QUEUE.as_view(),
        ),
        5 => reschedule(),
        _ => warn!("Unexpected syscall number: {}", syscall_number),
    }
}
