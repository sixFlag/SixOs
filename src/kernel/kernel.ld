OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    /* Link the kernel at this address: "." means the current address */
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    etext = .;

    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
    }
    . = ALIGN(4K);
    erodata = .;

    sdata = .;
    kernel_stack_bottom = .;
    .data : {
        *(.data.stack)
        . = ALIGN(4K);
        kernel_stack_top = .;
        *(.data .data.*)
    }
    . = ALIGN(4K);
    edata = .;

    sbss = .;
    .bss : {
        *(.bss .bss.*)
    }
    . = ALIGN(4K);
    ebss = .;

    ekernel = .;

	/DISCARD/ : {
		*(.eh_frame .eh_frame.*)
	}    

}