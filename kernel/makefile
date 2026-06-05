PROJECT=motossera-kernel


LUAVERSION = 5.5.0
LUATAR=lua-$(LUAVERSION).tar.gz
LUAURL=https://www.lua.org/ftp/$(LUATAR)

LIMINE_DIR = limine

all: kernel.iso

lua:
	if [ ! -d lua ]; then \
		curl -L -R -O $(LUAURL); \
		mkdir -p lua; \
		tar zxf $(LUATAR) -C lua --strip-components=1; \
	fi

limine:
	if [ ! -d limine ]; then \
		git clone https://github.com/limine-bootloader/limine.git; \
	fi
	cd limine && ./bootstrap
	cd limine && ./configure --enable-bios --enable-bios-cd --enable-uefi-x86-64 --enable-uefi-cd
	cd limine && make

kernel.bin: lua limine
	cargo build --release
	
	cp target/x86_64-unknown-none/release/$(PROJECT) kernel.bin

iso: kernel.bin
	mkdir -p iso/boot
	mkdir -p iso/EFI/BOOT

	cp kernel.bin iso/
	cp limine.conf iso/
	cp limine/bin/limine-bios.sys iso/
	cp limine/bin/limine-bios-cd.bin iso/
	cp limine/bin/limine-uefi-cd.bin iso/
	cp limine/bin/BOOTX64.EFI iso/EFI/BOOT/

kernel.iso: iso
	xorriso -as mkisofs \
		-b limine-bios-cd.bin \
		-no-emul-boot \
		-boot-load-size 4 \
		-boot-info-table \
		--efi-boot limine-uefi-cd.bin \
		-efi-boot-part \
		--efi-boot-image \
		--protective-msdos-label \
		iso -o kernel.iso
	./limine/bin/limine bios-install kernel.iso

run: kernel.iso
	qemu-system-x86_64 \
		-cdrom kernel.iso \
		-machine q35 \
		-serial stdio \
		-cpu host,+topoext \
		-smp 1 \
		-enable-kvm \
		-m 8G \
		-boot d

clean:
	rm -rf iso kernel.iso kernel.bin target/ *.o 
	cargo clean 

megaclean: clean
	rm -rf limine $(LUATAR) lua/ ftp 

comrun: clean run