pub mod mocks;
mod pci_tests;
mod sdhci_tests;
mod macros;

pub fn run_kernel_tests() {
    pci_tests::run_tests();
    sdhci_tests::run_tests();
}
