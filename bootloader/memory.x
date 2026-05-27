MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 64K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 192K
  CCMRAM (rwx) : ORIGIN = 0x10000000, LENGTH = 64K
}




/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);




 SECTIONS
  {
    .ccmram_fast (NOLOAD) :
    {
      . = ALIGN(4);
      _sccmram_fast = .;
      KEEP(*(.ccmram_fast .ccmram_fast.*));
      . = ALIGN(4);
      _eccmram_fast = .;
    } > CCMRAM

    .ccmram_text :
    {
      . = ALIGN(4);
      _sccmram_text = .;
      KEEP(*(.ccmram_text .ccmram_text.*));
      . = ALIGN(4);
      _eccmram_text = .;
    } > CCMRAM AT > FLASH

    _siccmram_text = LOADADDR(.ccmram_text);
  }
