MEMORY
{
  FLASH  : ORIGIN = 0x80200000, LENGTH = 1M     /* Kernel .text */
  RODATA : ORIGIN = 0x80300000, LENGTH = 1M     /* Kernel .rodata */
  RAM    : ORIGIN = 0x80400000, LENGTH = 2M     /* Kernel .data, .bss, heap */
  HEAP   : ORIGIN = 0x80700000, LENGTH = 2M
}

REGION_ALIAS("REGION_TEXT",   FLASH);
REGION_ALIAS("REGION_RODATA", RODATA);
REGION_ALIAS("REGION_DATA",   RAM);
REGION_ALIAS("REGION_BSS",    RAM);
REGION_ALIAS("REGION_HEAP",   HEAP);
REGION_ALIAS("REGION_STACK",  RAM);            /* Stack also in RAM */

_stack_start = ORIGIN(RAM) + LENGTH(RAM);      /* Top of RAM for stack */
_heap_size = 1M;
_hart_stack_size = 1K;
_max_hart_id = 1;
