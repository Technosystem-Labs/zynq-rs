ENTRY(_boot_cores);

STACK_SIZE = 0x2000 - 0x10;
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
  OCM : ORIGIN = 0, LENGTH = 0x40000
}

SECTIONS
{
    .exceptions :
    {
        . = 0x0;
        KEEP(*(.text.exceptions));
    } > OCM
    .text :
    {
        . = 0x8000;
        KEEP(*(.text.boot))
        *(.text .text.*)
        . = ALIGN(4096); /* align to page size */
    } > OCM
 
    .rodata :
    {
        *(.rodata)
        . = ALIGN(4096); /* align to page size */
    } > OCM
 
    .data :
    {
        *(.data)
        . = ALIGN(4096); /* align to page size */
    } > OCM
 
    .bss (NOLOAD) :
    {
        *(.bss)
        . = ALIGN(4096); /* align to page size */
    } > OCM
    __bss_start = ADDR(.bss);
    __bss_end = ADDR(.bss) + SIZEOF(.bss);

    .stack (NOLOAD) : {
      . += STACK_SIZE;
    } > OCM
    __stack_end = ADDR(.stack);
    __stack_start = ADDR(.stack) + SIZEOF(.stack);

  /DISCARD/ :
  {
    /* Unused exception related info that only wastes space */
    *(.ARM.exidx.*);
    *(.ARM.extab.*);
  }
}
