ENTRY(_boot_cores);

STACK_SIZE = 0x8000;

/* Provide some defaults */
PROVIDE(Reset = _boot_cores);
PROVIDE(UndefinedInstruction = Reset);
PROVIDE(SoftwareInterrupt = Reset);
PROVIDE(PrefetchAbort = Reset);
PROVIDE(DataAbort = Reset);
PROVIDE(ReservedException = Reset);
PROVIDE(IRQ = Reset);
PROVIDE(FIQ = Reset);

MEMORY
{
  /* 256 kB On-Chip Memory */
  OCM : ORIGIN = 0, LENGTH = 0x30000
  OCM3 : ORIGIN = 0xFFFF0000, LENGTH = 0x10000
}

SECTIONS
{
    .exceptions ORIGIN(OCM) :
    {
        KEEP(*(.text.exceptions));
    } > OCM

    .__fill (NOLOAD) : {
          . = ORIGIN(OCM) + 0x8000;
    } > OCM

    .text (ORIGIN(OCM) + 0x8000) :
    {
        *(.text.boot);
        *(.text .text.*);
    } > OCM
 
    .rodata : ALIGN(4)
    {
        *(.rodata .rodata.*);
    } > OCM
 
    .data : ALIGN(4)
    {
        *(.data .data.*);
    } > OCM
 
    .bss (NOLOAD) : ALIGN(0x4000)
    {
        /* Aligned to 16 kB */
        KEEP(*(.bss.l1_table));
        *(.bss .bss.*);
        . = ALIGN(4);
    } > OCM
    __bss_start = ADDR(.bss);
    __bss_end = ADDR(.bss) + SIZEOF(.bss);

    .stack (NOLOAD) : ALIGN(0x1000) {
      . += STACK_SIZE;
    } > OCM
    __stack_end = ADDR(.stack);
    __stack_start = ADDR(.stack) + SIZEOF(.stack) - 4;

  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx);
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}
