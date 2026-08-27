pub mod mocks;
mod pci_tests;

pub fn run_kernel_tests() {
    pci_tests::run_tests();
}
