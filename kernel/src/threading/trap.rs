pub enum ExceptionKind {
    Unknown,
}

pub enum TrapCause {
    Timer,
    SysCall{ syscall_number: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize },
    Exception(ExceptionKind),
    ExternalInterrupt(u8),
}

pub enum TrapHandlerRegistrationError {
    AlreadyOccupied,
}

