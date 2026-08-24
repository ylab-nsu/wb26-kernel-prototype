QEMU = "qemu-system-riscv64"
KERNEL = target/riscv64gc-unknown-none-elf/debug/kernel
APP_ELF = ../baremetal-test-exec/test/print/print.elf
# NOTE: file is loaded BELOW its linked vaddr (0x8b000000) on purpose,
# so the loader demonstrably copies segments to their runtime addresses.
APP_ADDR = 0x8A000000

.PHONY: run debug

run:
	$(QEMU) -kernel $(KERNEL) -machine virt -nographic -no-shutdown -serial mon:stdio \
	  -device loader,file=$(APP_ELF),addr=$(APP_ADDR),force-raw=on -m 256M

debug:
	$(QEMU) -kernel $(KERNEL) -machine virt -nographic -no-shutdown -serial mon:stdio -s -S \
	  -device loader,file=$(APP_ELF),addr=$(APP_ADDR),force-raw=on -m 256M