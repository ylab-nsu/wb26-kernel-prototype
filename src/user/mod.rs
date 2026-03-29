mod user1;
mod crt0;
mod og_processes;

pub use crt0::crt0;

pub struct UserProgram {
    pub entry: extern "C" fn(),
    pub stack_size: usize,
}

pub const USER_PROGRAMS: &[UserProgram] = &[
    UserProgram{entry: user1::main, stack_size:1024*1024},
    UserProgram{entry: og_processes::process1, stack_size:1024*1024},
    UserProgram{entry: og_processes::process2, stack_size:1024*1024},
    UserProgram{entry: og_processes::process3, stack_size:1024*1024},
];
