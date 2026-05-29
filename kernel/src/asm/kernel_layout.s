.section .rodata.kernel_layout

.global __kernel_layout
__kernel_layout:
    .dword __kernel_va_offset
    .dword __stext
    .dword __etext
    .dword __srodata
    .dword __erodata
    .dword __sdata
    .dword __edata
    .dword __sbss
    .dword __ebss
    .dword __spage_pool
    .dword __epage_pool
    .dword __user_va_offset
