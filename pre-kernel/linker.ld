ENTRY(_start)

SECTIONS {
	. = 0x500;

	.multiboot_header : {
		*(.multiboot_header)
	}
	. = 0x550;
	.start : {
		*(.start)
	}
	.text : {
		*(.text .text.*)
	}
	.bss : {
		*(.bss .bss.*)
	}
	.rodata : {
		*(.rodata .rodata.*)
		*(.comment .comment.*)
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
}
