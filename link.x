ENTRY(_boot_cores);

STACK_SIZE = 0x2000 - 8;

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
  OCM : ORIGIN = 0, LENGTH = 0x40000
}

SECTIONS
{
    .exceptions (0x0) :
    {
        KEEP(*(.text.exceptions));
    } > OCM
    .text (0x8000) :
    {
        KEEP(*(.text.boot))
        *(.text .text.*)
    } > OCM
 
    .rodata ALIGN(0x1000) :
    {
        *(.rodata)
    } > OCM
 
    .data ALIGN(0x1000) :
    {
        *(.data)
    } > OCM
 
    .bss ALIGN(0x1000) (NOLOAD) :
    {
        *(.bss)
    } > OCM
    __bss_start = ADDR(.bss);
    __bss_end = ADDR(.bss) + SIZEOF(.bss);

    .stack ALIGN(0x1000) (NOLOAD) : {
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
