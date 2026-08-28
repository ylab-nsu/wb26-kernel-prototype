QEMU = "qemu-system-riscv64"

.PHONY: run debug

run:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/kernel -machine virt -nographic -no-shutdown -serial mon:stdio

debug:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/kernel -machine virt -nographic -no-shutdown -serial mon:stdio -s -S
