QEMU = "qemu-system-riscv64"
TARGET = riscv64gc-unknown-none-elf
QEMU_FLAGS = -machine virt -nographic -no-shutdown -serial mon:stdio

KERNEL_DEBUG = target/$(TARGET)/debug/kernel
KERNEL_RELEASE = target/$(TARGET)/release/kernel

.PHONY: run run-release debug debug-release

run:
	$(QEMU) -kernel $(KERNEL_DEBUG) $(QEMU_FLAGS)

run-release:
	$(QEMU) -kernel $(KERNEL_RELEASE) $(QEMU_FLAGS)

debug:
	$(QEMU) -kernel $(KERNEL_DEBUG) $(QEMU_FLAGS) -s -S

debug-release:
	$(QEMU) -kernel $(KERNEL_RELEASE) $(QEMU_FLAGS) -s -S
