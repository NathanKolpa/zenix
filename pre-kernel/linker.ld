ENTRY(_start)

SECTIONS {
	. = 1M;

	STACK_START = .;
	. = . + 128K;
	STACK_END = .;

	. = . + 4;

	BUMP_MEMORY_START = .;

	. = 3M;
	BUMP_MEMORY_END = .;

	PRE_KERNEL_START = .;
	.text ALIGN(0x10) : {
		*(.multiboot_header)
		*(.start)
		*(.text .text.*)
	}
	.bss : {
		*(.bss .bss.*)
	}
	.rodata : {
		*(.rodata .rodata.*)
		*(.comment .comment.*)
	}
	.eh_frame : {
		*(.eh_frame .eh_frame.*)
	}
	.eh_frame_hdr : {
		*(.eh_frame_hdr .eh_frame_hdr.*)
	}
	.debug : {
		*(.debug_loc .debug_loc.*)
		*(.debug_abbrev .debug_abbrev.*)
		*(.debug_info .debug_info.*)
		*(.debug_aranges .debug_aranges.*)
		*(.debug_ranges .debug_ranges.*)
		*(.debug_str .debug_str.*)
		*(.debug_frame .debug_frame.*)
		*(.debug_line .debug_line.*)
		*(.debug_line .debug_line.*)
	}

	PRE_KERNEL_END = .;
	
}
