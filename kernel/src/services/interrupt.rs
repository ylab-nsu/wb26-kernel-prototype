use crate::arch::threading::trap::{
    ExceptionKind,
    TrapCause,
    SysCallContext,
};
use crate::sync::InterruptMutex;
use alloc::{ boxed::Box, sync::Arc };

type TimerHandler = dyn Fn() -> bool + Send + Sync;
type SysCallHandler = dyn Fn(SysCallContext) -> () + Send + Sync;
type ExceptionHandler = dyn Fn(ExceptionKind) -> () + Send + Sync;
type ExternalInterruptHandler = dyn Fn() -> () + Send + Sync;

pub enum TrapHandler {
    Timer(Box<TimerHandler>),
    SysCall(Box<SysCallHandler>),
    Exception(Box<ExceptionHandler>),
    ExternalInterrupt{ irq: u8, handler: Box<ExternalInterruptHandler> },
}

pub struct InterruptService {
    timer_handler: InterruptMutex<Option<Arc<TimerHandler>>>,
    syscall_handler: InterruptMutex<Option<Arc<SysCallHandler>>>,
    exception_handler: InterruptMutex<Option<Arc<ExceptionHandler>>>,
    external_int_handlers: [InterruptMutex<Option<Arc<ExternalInterruptHandler>>>; 256],
}

pub enum TrapHandlerGuard<'a> {
    Timer(&'a InterruptMutex<Option<Arc<TimerHandler>>>),
    SysCall(&'a InterruptMutex<Option<Arc<SysCallHandler>>>),
    Exception(&'a InterruptMutex<Option<Arc<ExceptionHandler>>>),
    ExternalInterrupt(&'a InterruptMutex<Option<Arc<ExternalInterruptHandler>>>),
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
                        *guard = Some(Arc::from(handler));
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
                        *guard = Some(Arc::from(handler));
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
                        *guard = Some(Arc::from(handler));
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
                        *guard = Some(Arc::from(handler));
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
                let maybe_arc_handler = {
                    (*self.timer_handler.lock()).clone()
                };

                maybe_arc_handler.map_or(false, |arc_handler| (*arc_handler)())
            },
            TrapCause::SysCall(ctx) => {
                let maybe_arc_handler = {
                    (*self.syscall_handler.lock()).clone()
                };
                if let Some(arc_handler) = maybe_arc_handler {
                    (*arc_handler)(ctx);
                }
                false
            },
            TrapCause::Exception(kind) => {
                let maybe_arc_handler = {
                    (*self.exception_handler.lock()).clone()
                };
                if let Some(arc_handler) = maybe_arc_handler {
                    (*arc_handler)(kind);
                }
                false
            },
            TrapCause::ExternalInterrupt(irq) => {
                let maybe_arc_handler = {
                    (*self.external_int_handlers[irq as usize].lock()).clone()
                };
                if let Some(arc_handler) = maybe_arc_handler {
                    (*arc_handler)();
                }
                false
            },
        }
    }
}
