.section .multiboot_header
	.balign 4
	.set MULTIBOOT_HEADER_MAGIC, 0x1BADB002

	.set MULTIBOOT_PAGE_ALIGN, 1<<0
	.set MULTIBOOT_MEMORY_INFO, 1<<1
	.set MULTIBOOT_HEADER_FLAGS, (MULTIBOOT_MEMORY_INFO | MULTIBOOT_PAGE_ALIGN)

header_start:
	.long MULTIBOOT_HEADER_MAGIC
	.long MULTIBOOT_HEADER_FLAGS
	.long  -(MULTIBOOT_HEADER_MAGIC + MULTIBOOT_HEADER_FLAGS) // checksum
header_end:

.section .bss
	.align 16
	STACK_BOTTOM:
	.skip 16384 // 16 KiB
	STACK_TOP:

.code32 // < very important!!
.section .text
.global _start
.type _start, @function
_start:

	// setup the stack
	mov esp, STACK_TOP
	mov ebp, STACK_TOP
	
	// push the magic to arg 0
	// From the spec [EAX]: Must contain the magic value ‘0x2BADB002’; the presence of this value indicates to the operating system that it was loaded by a Multiboot-compliant boot loader (e.g. as opposed to another type of boot loader that the operating system can also be loaded from).
	mov ecx, eax

	// push the addr to arg 1
	// From the spec: [EBX]: Must contain the 32-bit physical address of the Multiboot information structure provided by the boot loader (see Boot information format).
	mov edx, ebx

	cld

	// Go to rust land!
	call main


	// Prevent the cpu from executing memory.
	// This code should never be executed anyways.
	cli
	hlt
