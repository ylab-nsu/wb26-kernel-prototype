QEMU = "qemu-system-riscv64"

.PHONY: run debug

run:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/kernel -machine virt \
			-nographic -no-shutdown -serial mon:stdio \
			-device sdhci-pci \
			-drive file=emmc.img,if=none,format=raw,id=emmc-img \
			-device emmc,boot-partition-size=0,rpmb-partition-size=0,drive=emmc-img

debug:
	$(QEMU) -kernel target/riscv64gc-unknown-none-elf/debug/kernel -machine virt -nographic -no-shutdown -serial mon:stdio -s -S
