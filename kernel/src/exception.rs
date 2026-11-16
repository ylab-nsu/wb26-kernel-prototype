use crate::arch::threading::trap::ExceptionKind;

pub fn handle_exception(e: ExceptionKind) {
    match e {
        ExceptionKind::Unknown => panic!("Unknown exception occured"),
        ExceptionKind::InstructionFault{ address } => panic!("InstructionFault {address:x}"),
    };
}
