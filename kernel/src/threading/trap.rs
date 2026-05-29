use riscv::interrupt::{Exception, Interrupt, Trap};
use crate::drivers::{put_into_queue, TestDriverMessage, TEST_DRIVER_QUEUE};

#[export_name = "_handle_trap_rust"]
extern "C" fn handle_trap(frame: &mut crate::threading::thread::TrapFrame) -> bool {
    // println!("Current SP: {:p}", frame);
    let x: Trap<Interrupt, Exception> = riscv::register::scause::read().cause().try_into().unwrap();
    println!("Cause: {x:?}");

    let mut need_reschedule = false;

    match x {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            need_reschedule = true;
        }

        Trap::Exception(Exception::UserEnvCall) => {
            if frame.a7 < (('A' as usize) * 256) {
                // println!("Received non-SBI UserEnvCall");
                handle_syscall(frame);
            } else {
                // println!("    Redirecting UserEnvCall to SBI");
                unsafe {
                    core::arch::asm!(
                    "ecall",
                    inlateout("a0") frame.a0 => frame.a0,
                    inlateout("a1") frame.a1 => frame.a1,
                    in("a2") frame.a2,
                    in("a3") frame.a3,
                    in("a4") frame.a4,
                    in("a5") frame.a5,
                    in("a6") frame.a6,
                    in("a7") frame.a7,
                    options(nostack),
                    );
                }
            }
            frame.pc += 4;
        }

        Trap::Exception(Exception::InstructionFault) => {
            sbi::timer::set_timer(u64::MAX).expect("Can't set timer");
            let epc = riscv::register::sepc::read();
            panic!("InstructionFault {epc:x} {}", frame.pc);
        }

        Trap::Interrupt(cause) => {
            println!("interrupt:{cause:?}");
        }
        Trap::Exception(cause) => {
            println!("exception:{cause:?}");
        }
    }
    need_reschedule
}

fn handle_syscall(frame: &mut crate::threading::thread::TrapFrame) {
    match frame.a7 {
        1 => put_into_queue(
            TestDriverMessage::PrintNumber { number: frame.a0 },
            TEST_DRIVER_QUEUE.as_view(),
        ),
        2 => put_into_queue(
            TestDriverMessage::PrintString {
                user_addr: frame.a0,
                len: frame.a1,
            },
            TEST_DRIVER_QUEUE.as_view(),
        ),
        _ => println!("Unexpected syscall number: {}", frame.a7),
    }
}
