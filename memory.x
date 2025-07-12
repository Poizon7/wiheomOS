MEMORY
{
  FLASH : ORIGIN = 0x80200000, LENGTH = 2M     /* Kernel .text and .rodata */
  RAM   : ORIGIN = 0x80400000, LENGTH = 2M     /* Kernel .data, .bss, heap */
}

REGION_ALIAS("REGION_TEXT",   FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA",   RAM);
REGION_ALIAS("REGION_BSS",    RAM);
REGION_ALIAS("REGION_HEAP",   RAM);
REGION_ALIAS("REGION_STACK",  RAM);            /* Stack also in RAM */

_stack_start = ORIGIN(RAM) + LENGTH(RAM);      /* Top of RAM for stack */
_heap_size = 1K;
_hart_stack_size = 1K;
_max_hart_id = 1;
