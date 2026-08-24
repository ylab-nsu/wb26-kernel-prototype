QEMU = "qemu-system-riscv64"

.PHONY: run debug

run:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/kernel -machine virt -nographic -no-shutdown -serial mon:stdio 

debug:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/kernel -machine virt -nographic -no-shutdown -serial mon:stdio -s -S

release:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/release/kernel -machine virt -nographic -no-shutdown -serial mon:stdio

rx-test:
	python3 -c 'import sys; sys.stdout.write("A" * 14); sys.stdout.flush()' | \
	$(QEMU) \
		-kernel target/riscv64gc-unknown-none-elf/debug/kernel \
		-machine virt \
		-nographic \
		-no-shutdown \
		-serial stdio