pub mod exec_tests;
pub mod macros;
pub mod mocks;

pub fn run_kernel_tests() {
    exec_tests::run_tests();
}