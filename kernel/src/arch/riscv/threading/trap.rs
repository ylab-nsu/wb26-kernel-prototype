use crate::{
    arch::traits::{TargetTrapFrame, TargetInterruptController},
    threading::trap::{TrapCause, TrapHandlerRegistrationError, ExceptionKind},
};
use once_cell_no_std::OnceCell;
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

static KERNEL_TRAP_HANDLER: OnceCell<fn(TrapCause) -> bool> = OnceCell::new();

pub struct RiscvInterruptController;

impl TargetInterruptController for RiscvInterruptController {
    fn set_kernel_trap_handler(handler: fn(TrapCause) -> bool) -> Result<(), TrapHandlerRegistrationError> {
        return KERNEL_TRAP_HANDLER.set(handler).map_err(|_| TrapHandlerRegistrationError::AlreadyOccupied);
    }
}

pub fn setup_trap() {
    extern "C" {
        fn _start_trap();
    }
    unsafe {
        riscv::register::stvec::write(Stvec::new(
            _start_trap as *const () as usize,
            TrapMode::Direct,
        ))
    }
}

#[export_name = "_handle_trap_rust"]
extern "C" fn handle_trap(frame: &mut RiscvTrapFrame) -> bool {
    let x: Trap<Interrupt, Exception> = riscv::register::scause::read().cause().try_into().unwrap();

    // Special bypass for direct SBI calls such as sbi_print!
    // Useful to print information immediately instead of waiting in queue for driver
    if matches!(x, Trap::Exception(Exception::UserEnvCall)) && frame.a7 >= (('A' as usize) * 256) {
        // println!("Redirecting UserEnvCall to SBI");
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
        frame.pc += 4;
        return false;
    }

    info!("Trap cause: {x:?}");

    if let Some(handler) = KERNEL_TRAP_HANDLER.get() {
        match x {
            Trap::Interrupt(Interrupt::SupervisorTimer) => {
                handler(TrapCause::Timer)
            },
            Trap::Exception(Exception::UserEnvCall) => {
                // println!("Received non-SBI UserEnvCall");
                let ret = handler(TrapCause::SysCall {
                        syscall_number: frame.a7,
                        arg1: frame.a0, 
                        arg2: frame.a1, 
                        arg3: frame.a2, 
                        arg4: frame.a3,
                });
                frame.pc += 4;

                ret
            }

            Trap::Exception(Exception::InstructionFault) => {
                sbi::timer::set_timer(u64::MAX).expect("Can't set timer");
                let epc = riscv::register::sepc::read();
                error!("InstructionFault {epc:x} {}", frame.pc);

                handler(TrapCause::Exception(ExceptionKind::InstructionFault{ address: epc }))
            }

            Trap::Interrupt(cause) => {
                println!("interrupt:{cause:?}");
                false
            }
            Trap::Exception(cause) => {
                println!("exception:{cause:?}");
                false
            }
        }
    } else { false }
}
