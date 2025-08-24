ENTRY( _start )

MEMORY
{
  ram   (wxa!ri) : ORIGIN = 0x80000000, LENGTH = 256K
}

SECTIONS
{
    .text :
    {
        *(.text*)
        *(.rodata*)
    } > ram

    /* .bss section which is used for uninitialized data */
    .bss (NOLOAD) :
    {
        *(.bss*)
        *(COMMON)
    } > ram

    .data :
    {
        *(.data*);
    } > ram


    PROVIDE(_memory_start = ORIGIN(ram));
    PROVIDE(_memory_end = ORIGIN(ram) + LENGTH(ram));
    end = .;
}
