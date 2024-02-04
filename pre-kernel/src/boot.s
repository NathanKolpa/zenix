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
	.set STACK_SIZE, 16384

	.align 16
	STACK_BOTTOM:
	.skip STACK_SIZE
	STACK_TOP:
	.skip 4

.code32
.section .text
.global _start
.type _start, @function
_start:
	// From the spec [EAX]: Must contain the magic value '0x2BADB002'; the presence of this value indicates to the operating system that it was loaded by a Multiboot-compliant boot loader (e.g. as opposed to another type of boot loader that the operating system can also be loaded from).

	mov [STACK_TOP], eax
	mov [STACK_TOP - 4], ebx
	// From the spec: [EBX]: Must contain the 32-bit physical address of the Multiboot information structure provided by the boot loader (see Boot information format).

	cld

	/// setup the stack
	mov     eax, 1
	cpuid
	shr     ebx, 24
	add     ebx, 1
	mov     eax, STACK_SIZE
	mul     ebx
	add     eax, STACK_BOTTOM
	mov     esp, eax
	
	mov eax, DWORD PTR [STACK_TOP]
	mov ebx, DWORD PTR [STACK_TOP - 4]
	push ebx
	push eax

	// Go to rust land!
	call main


	// Prevent the cpu from executing memory.
	// This code should never be executed anyways.
	cli
	hlt
