QEMU = "qemu"

.PHONY: run debug

run:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/risc-v-rust-bare-metal -machine virt -no-shutdown -serial mon:stdio

debug:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/risc-v-rust-bare-metal -machine virt -no-shutdown -serial mon:stdio -s -S
