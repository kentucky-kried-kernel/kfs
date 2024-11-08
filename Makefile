SRC_DIR := src
OBJ_DIR := obj
BUILD_DIR := build
NAME := kernel
BINARY := $(NAME).bin
ISO := $(NAME).iso
MULTIBOOT_HEADER := assets/boot.s
MULTIBOOT_HEADER_OBJ := boot.o

LIB := target/i386-unknown-none/release/libkfs.a

RUST_SRCS := $(shell find $(SRC_DIR) -type f -name "*.rs")
CARGO_TOML := Cargo.toml

all: $(BUILD_DIR)/$(BINARY)

$(BUILD_DIR)/$(BINARY): $(BUILD_DIR)/$(MULTIBOOT_HEADER_OBJ) $(LIB)
	ld -m elf_i386 -T assets/linker.ld -o $@ $^

$(BUILD_DIR)/$(MULTIBOOT_HEADER_OBJ): $(MULTIBOOT_HEADER) | $(BUILD_DIR)
	as --32 -o $@ $<

$(LIB): $(RUST_SRCS) $(CARGO_TOML) $(MULTIBOOT_HEADER)
	cargo build-kernel
	touch $(LIB)

$(BUILD_DIR):
	mkdir -p $@

iso: all 
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp assets/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso --compress=xz

run: iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -curses

debug: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp assets/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d



debug_terminal: all
	mkdir -p $(BUILD_DIR)/iso/boot/grub
	cp assets/grub.cfg $(BUILD_DIR)/iso/boot/grub
	cp $(BUILD_DIR)/kernel.bin $(BUILD_DIR)/iso/boot/
	grub-mkrescue -v -o $(BUILD_DIR)/$(NAME).iso $(BUILD_DIR)/iso
	qemu-system-i386 -cdrom $(BUILD_DIR)/$(NAME).iso -boot d -curses

fclean:
	cargo clean
	$(RM) -rf $(BUILD_DIR)

re: fclean all

.PHONY: all run re fclean iso debug
