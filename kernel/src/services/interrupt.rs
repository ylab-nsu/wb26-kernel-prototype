use crate::threading::trap::{
    TrapHandlerRegistrationError,
    ExceptionKind,
    TrapCause,
};
use crate::arch::{
    InterruptController,
    traits::TargetInterruptController,
};
use core::sync::atomic::Ordering;
use atomic_fn::AtomicFnPtr;
use once_cell_no_std::OnceCell;

type TimerHandler = fn() -> bool;
type SysCallHandler = fn(usize, usize, usize, usize, usize) -> ();
type ExceptionHandler = fn(ExceptionKind) -> ();
type ExternalInterruptHandler = fn() -> ();

pub enum TrapHandler {
    Timer(TimerHandler),
    SysCall(SysCallHandler),
    Exception(ExceptionHandler),
    ExternalInterrupt{ irq: u8, handler: ExternalInterruptHandler },
}

pub struct InterruptService {
    timer_handler: AtomicFnPtr<Option<TimerHandler>>,
    syscall_handler: AtomicFnPtr<Option<SysCallHandler>>,
    exception_handler: AtomicFnPtr<Option<ExceptionHandler>>,
    external_int_handlers: [AtomicFnPtr<Option<ExternalInterruptHandler>>; 256],
}

pub enum TrapHandlerGuard<'a> {
    Timer(&'a AtomicFnPtr<Option<TimerHandler>>),
    SysCall(&'a AtomicFnPtr<Option<SysCallHandler>>),
    Exception(&'a AtomicFnPtr<Option<ExceptionHandler>>),
    ExternalInterrupt(&'a AtomicFnPtr<Option<ExternalInterruptHandler>>),
}

impl Drop for TrapHandlerGuard<'_> {
    fn drop(&mut self) {
        match self {
            TrapHandlerGuard::Timer(atom) => atom.store(None, Ordering::Relaxed),
            TrapHandlerGuard::SysCall(atom) => atom.store(None, Ordering::Relaxed),
            TrapHandlerGuard::Exception(atom) => atom.store(None, Ordering::Relaxed),
            TrapHandlerGuard::ExternalInterrupt(atom) => atom.store(None, Ordering::Relaxed),
        }
    }
}

impl InterruptService {
    fn new() -> Self {
        InterruptService {
            timer_handler: AtomicFnPtr::new(None),
            syscall_handler: AtomicFnPtr::new(None),
            exception_handler: AtomicFnPtr::new(None),
            external_int_handlers: core::array::from_fn(|_| AtomicFnPtr::new(None)),
        }
    }

    pub fn register_trap_handler(&self, handler: TrapHandler) -> Result<TrapHandlerGuard<'_>, TrapHandlerRegistrationError> {
        match handler {
            TrapHandler::Timer(handler) => {
                self.timer_handler
                    .compare_exchange(None, Some(handler), Ordering::Relaxed, Ordering::Relaxed)
                    .map(|_| TrapHandlerGuard::Timer(&self.timer_handler))
                    .map_err(|_| TrapHandlerRegistrationError::AlreadyOccupied)
            },
            TrapHandler::SysCall(handler) => {
                self.syscall_handler
                    .compare_exchange(None, Some(handler), Ordering::Relaxed, Ordering::Relaxed)
                    .map(|_| TrapHandlerGuard::SysCall(&self.syscall_handler))
                    .map_err(|_| TrapHandlerRegistrationError::AlreadyOccupied)
            },
            TrapHandler::Exception(handler) => {
                self.exception_handler
                    .compare_exchange(None, Some(handler), Ordering::Relaxed, Ordering::Relaxed)
                    .map(|_| TrapHandlerGuard::Exception(&self.exception_handler))
                    .map_err(|_| TrapHandlerRegistrationError::AlreadyOccupied)
            },
            TrapHandler::ExternalInterrupt{ irq, handler } => {
                self.external_int_handlers[irq as usize]
                    .compare_exchange(None, Some(handler), Ordering::Relaxed, Ordering::Relaxed)
                    .map(|_| TrapHandlerGuard::ExternalInterrupt(&self.external_int_handlers[irq as usize]))
                    .map_err(|_| TrapHandlerRegistrationError::AlreadyOccupied)
            },
        }
    }

    pub fn handle_trap(&self, trap: TrapCause) -> bool {
        match trap {
            TrapCause::Timer => {
                self.timer_handler.load(Ordering::Relaxed).map_or(false, |handler| handler())
            },
            TrapCause::SysCall{ syscall_number, arg1, arg2, arg3, arg4 } => {
                if let Some(handler) = self.syscall_handler.load(Ordering::Relaxed) {
                    handler(syscall_number, arg1, arg2, arg3, arg4);
                }
                false
            },
            TrapCause::Exception(kind) => {
                if let Some(handler) = self.exception_handler.load(Ordering::Relaxed) {
                    handler(kind);
                }
                false
            },
            TrapCause::ExternalInterrupt(irq) => {
                if let Some(handler) = self.external_int_handlers[irq as usize].load(Ordering::Relaxed) {
                    handler();
                }
                false
            },
        }
    }
}

#[derive(Debug)]
pub enum InterruptServiceInitializationError {
    AlreadyInited,
    AnotherKernelTrapHandlerAlreadySet,
}

static INTERRUPT_SERVICE: OnceCell<InterruptService> = OnceCell::new();

pub fn kernel_trap_handler(trap: TrapCause) -> bool {
    INTERRUPT_SERVICE.get().map_or(false, |service| service.handle_trap(trap))
}

pub fn init() -> Result<&'static InterruptService, InterruptServiceInitializationError> {
    INTERRUPT_SERVICE.set(InterruptService::new()).map_err(|_| InterruptServiceInitializationError::AlreadyInited)?;

    InterruptController::set_kernel_trap_handler(kernel_trap_handler).map_err(|_| InterruptServiceInitializationError::AnotherKernelTrapHandlerAlreadySet)?;

    // SAFETY: If first statement returned Ok - the cell must be non-empty
    Ok(INTERRUPT_SERVICE.get().unwrap())
}
