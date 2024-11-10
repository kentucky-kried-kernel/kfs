.extern kernel_main

.global _start
.global stack_top
.global GDT
.global GDT_LIMIT


.set MB_MAGIC, 0x1BADB002          				# This magic number is used for validating the boot code - https://en.wikipedia.org/wiki/Magic_number_(programming)
.set MB_FLAGS, 0								# Flags - 0 == protected boot mode
.set MB_CHECKSUM, (0 - (MB_MAGIC + MB_FLAGS))	# Needed for grub multiboot header verification

.section .multiboot
	.align 4 			# Aligns further values so that - ADDR_OF_VAR % 4 == 0
	.long MB_MAGIC		# .long == u32 allocated with value of MB_MAGIC
	.long MB_FLAGS
	.long MB_CHECKSUM

.section .bss			# .bss is used for all variables that are already declared but uninitialized - stack & global/static variables

	.align 16			# Aligns further values so that - ADDR_OF_VAR % 4 == 0
	.skip 4096		# .skip tells the assemler to not put something for 4096 bytes
	stack_top:			# Saves the address into stack_top

.section .data
	gdtr:	.word 0x0
			.long 0x0

.section .text
	_start:
		mov $stack_top, %esp
	
	_gdt:
		xor		%eax, %eax
		lea		GDT, %eax 
		mov		%eax, (gdtr + 2)
		mov 	GDT_LIMIT, %eax
		mov		%eax, (gdtr)
		LGDT	gdtr

        push %eax
        push %ebx
		cli
		call kernel_main
 
		hang:
			cli      			# disable all interups	
			hlt      			# halt - basically sleep and do nothing
			jmp hang			# normally with interupts disabled hlt should never finish except for non-maskable interrupt occurring or due to system management mode
