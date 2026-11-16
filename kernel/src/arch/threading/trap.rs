
pub enum ExceptionKind {
    InstructionFault{ address: usize },
    Unknown,
}

pub struct SysCallContext {
    pub syscall_number: usize,
    pub arg1: usize,
    pub arg2: usize,
    pub arg3: usize,
    pub arg4: usize,
}

pub enum TrapCause {
    Timer,
    SysCall(SysCallContext),
    Exception(ExceptionKind),
    ExternalInterrupt(u8),
}

#[derive(Debug)]
pub enum GlobalTrapHandlerRegistrationError {
    AlreadyOccupied,
}

pub type GlobalTrapHandler = dyn Fn(TrapCause) -> bool + Send + Sync;
