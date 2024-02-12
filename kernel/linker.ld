ENTRY(kernel_main)

SECTIONS {
	. = 5M;

	.text ALIGN(0x10) : {
		*(.text .text.*)
	}
	.data : {
		*(.data .data.*)
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
}
