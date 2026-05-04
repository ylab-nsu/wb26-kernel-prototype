use crate::arch::traits::TargetTrapFrame;
use riscv::interrupt::{Exception, Interrupt, Trap};
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec::Stvec;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct RiscvTrapFrame {
    pc: usize,
    ra: usize,
    sp: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    s0: usize,
    s1: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
}

impl TargetTrapFrame for RiscvTrapFrame {
    fn with_pc(mut self, pc: usize) -> Self {
        self.pc = pc;
        self
    }

    fn with_sp(mut self, sp: usize) -> Self {
        self.sp = sp;
        self
    }

    fn set_arg0(&mut self, value: usize) {
        self.a0 = value;
    }
}

pub fn setup_trap() {
    extern "C" {
        fn _start_trap();
    }
    unsafe { riscv::register::stvec::write(Stvec::new(_start_trap as usize, TrapMode::Direct)) }
}

#[export_name = "_handle_trap_rust"]
extern "C" fn handle_trap(frame: &mut RiscvTrapFrame) -> bool {
    // println!("Current SP: {:p}", frame);
    let x: Trap<Interrupt, Exception> = riscv::register::scause::read().cause().try_into().unwrap();
    println!("Cause: {x:?}");

    let mut need_reschedule = false;

    match x {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            need_reschedule = true;
        }

        Trap::Exception(Exception::UserEnvCall) => {
            if (frame.a6 == 0) {
                println!("Received non-SBI UserEnvCall");
            } else {
                println!("    Redirecting UserEnvCall to SBI");
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
                    );
                }
            }
            frame.pc += 4;
        }

        Trap::Exception(Exception::InstructionFault) => {
            sbi::timer::set_timer(u64::MAX).expect("Can't set timer");
            let epc = unsafe { riscv::register::sepc::read() };
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
