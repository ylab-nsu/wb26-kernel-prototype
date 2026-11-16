use crate::arch::threading::trap::{
    ExceptionKind,
    TrapCause,
};
use crate::sync::InterruptMutex;
use alloc::boxed::Box;

type TimerHandler = dyn Fn() -> bool + Send + Sync;
type SysCallHandler = dyn Fn(usize, usize, usize, usize, usize) -> () + Send + Sync;
type ExceptionHandler = dyn Fn(ExceptionKind) -> () + Send + Sync;
type ExternalInterruptHandler = dyn Fn() -> () + Send + Sync;

pub enum TrapHandler {
    Timer(Box<TimerHandler>),
    SysCall(Box<SysCallHandler>),
    Exception(Box<ExceptionHandler>),
    ExternalInterrupt{ irq: u8, handler: Box<ExternalInterruptHandler> },
}

pub struct InterruptService {
    timer_handler: InterruptMutex<Option<Box<TimerHandler>>>,
    syscall_handler: InterruptMutex<Option<Box<SysCallHandler>>>,
    exception_handler: InterruptMutex<Option<Box<ExceptionHandler>>>,
    external_int_handlers: [InterruptMutex<Option<Box<ExternalInterruptHandler>>>; 256],
}

pub enum TrapHandlerGuard<'a> {
    Timer(&'a InterruptMutex<Option<Box<TimerHandler>>>),
    SysCall(&'a InterruptMutex<Option<Box<SysCallHandler>>>),
    Exception(&'a InterruptMutex<Option<Box<ExceptionHandler>>>),
    ExternalInterrupt(&'a InterruptMutex<Option<Box<ExternalInterruptHandler>>>),
}

#[derive(Debug)]
pub enum TrapHandlerRegistrationError {
    AlreadyOccupied,
}

impl Drop for TrapHandlerGuard<'_> {
    fn drop(&mut self) {
        match self {
            TrapHandlerGuard::Timer(mtx) => *mtx.lock() = None,
            TrapHandlerGuard::SysCall(mtx) => *mtx.lock() = None,
            TrapHandlerGuard::Exception(mtx) => *mtx.lock() = None,
            TrapHandlerGuard::ExternalInterrupt(mtx) => *mtx.lock() = None,
        };
    }
}

impl InterruptService {
    pub const fn new() -> Self {
        InterruptService {
            timer_handler: InterruptMutex::new(None),
            syscall_handler: InterruptMutex::new(None),
            exception_handler: InterruptMutex::new(None),
            external_int_handlers: [const { InterruptMutex::new(None) }; 256],
        }
    }

    pub fn register_trap_handler(&self, handler: TrapHandler) -> Result<TrapHandlerGuard<'_>, TrapHandlerRegistrationError> {
        match handler {
            TrapHandler::Timer(handler) => {
                let mut guard = self.timer_handler.lock();
                match *guard {
                    None => {
                        *guard = Some(handler);
                        Ok(TrapHandlerGuard::Timer(&self.timer_handler))
                    },
                    Some(_) => {
                        Err(TrapHandlerRegistrationError::AlreadyOccupied)
                    },
                }
            },
            TrapHandler::SysCall(handler) => {
                let mut guard = self.syscall_handler.lock();
                match *guard {
                    None => {
                        *guard = Some(handler);
                        Ok(TrapHandlerGuard::SysCall(&self.syscall_handler))
                    },
                    Some(_) => {
                        Err(TrapHandlerRegistrationError::AlreadyOccupied)
                    },
                }
            },
            TrapHandler::Exception(handler) => {
                let mut guard = self.exception_handler.lock();
                match *guard {
                    None => {
                        *guard = Some(handler);
                        Ok(TrapHandlerGuard::Exception(&self.exception_handler))
                    },
                    Some(_) => {
                        Err(TrapHandlerRegistrationError::AlreadyOccupied)
                    },
                }
            },
            TrapHandler::ExternalInterrupt{ irq, handler } => {
                let mut guard = self.external_int_handlers[irq as usize].lock();
                match *guard {
                    None => {
                        *guard = Some(handler);
                        Ok(TrapHandlerGuard::ExternalInterrupt(&self.external_int_handlers[irq as usize]))
                    },
                    Some(_) => {
                        Err(TrapHandlerRegistrationError::AlreadyOccupied)
                    },
                }
            },
        }
    }

    pub fn handle_trap(&self, trap: TrapCause) -> bool {
        match trap {
            TrapCause::Timer => {
                let guard = self.timer_handler.lock();
                (*guard).as_ref().map_or(false, |boxedHandler| (*boxedHandler)())
            },
            TrapCause::SysCall{ syscall_number, arg1, arg2, arg3, arg4 } => {
                let guard = self.syscall_handler.lock();
                if let Some(boxedHandler) = &*guard {
                    (*boxedHandler)(syscall_number, arg1, arg2, arg3, arg4);
                }
                false
            },
            TrapCause::Exception(kind) => {
                let guard = self.exception_handler.lock();
                if let Some(boxedHandler) = &*guard {
                    (*boxedHandler)(kind);
                }
                false
            },
            TrapCause::ExternalInterrupt(irq) => {
                let guard = self.external_int_handlers[irq as usize].lock();
                if let Some(boxedHandler) = &*guard {
                    (*boxedHandler)();
                }
                false
            },
        }
    }
}
