
pub enum ExceptionKind {
    InstructionFault{ address: usize },
    Unknown,
}

pub enum TrapCause {
    Timer,
    SysCall{ syscall_number: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize },
    Exception(ExceptionKind),
    ExternalInterrupt(u8),
}

#[derive(Debug)]
pub enum GlobalTrapHandlerRegistrationError {
    AlreadyOccupied,
}

pub type GlobalTrapHandler = dyn Fn(TrapCause) -> bool + Send + Sync + 'static;
