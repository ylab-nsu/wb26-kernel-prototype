/* # Developer notes

- Symbols that start with a double underscore (__) are considered "private"

- Symbols that start with a single underscore (_) are considered "semi-public"; they can be
  overridden in a user linker script, but should not be referred from user code (e.g. `extern "C" {
  static mut _heap_size }`).

- `EXTERN` forces the linker to keep a symbol in the final binary. We use this to make sure a
  symbol is not dropped if it appears in or near the front of the linker arguments and "it's not
  needed" by any of the preceding objects (linker arguments)

- `PROVIDE` is used to provide default values that can be overridden by a user linker script

- In this linker script, you may find symbols that look like `${...}` (e.g., `8`).
  These are wildcards used by the `build.rs` script to adapt to different target particularities.
  Check `build.rs` for more details about these symbols.

- On alignment: it's important for correctness that the VMA boundaries of both .bss and .data *and*
  the LMA of .data are all `8`-byte aligned. These alignments are assumed by the RAM
  initialization routine. There's also a second benefit: `8`-byte aligned boundaries
  means that you won't see "Address (..) is out of bounds" in the disassembly produced by `objdump`.
*/

/* Default abort entry point. If no abort symbol is provided, then abort maps to _default_abort. */
/* EXTERN(_default_abort);
PROVIDE(abort = _default_abort); */

/* Trap for exceptions triggered during initialization. If the execution reaches this point, it
   means that there is a bug in the boot code. If no _pre_init_trap symbol is provided, then
  _pre_init_trap defaults to _default_abort. Note that _pre_init_trap must be 4-byte aligned */
/* PROVIDE(_pre_init_trap = _default_abort); */

/* Multi-processor hook function (for multi-core targets only). If no _mp_hook symbol
   is provided, then _mp_hook maps to _default_mp_hook, which leaves HART 0 running while
   the other HARTS stuck in a busy loop. Note that _default_mp_hook cannot be overwritten.
   We use PROVIDE to avoid compilation errors in single hart targets, not to allow users
   to overwrite the symbol. */
/* PROVIDE(_default_mp_hook = abort);
PROVIDE(_mp_hook = _default_mp_hook); */

/* Default trap entry point. If not _start_trap symbol is provided, then _start_trap maps to
   _default_start_trap, which saves caller saved registers, calls _start_trap_rust, restores
   caller saved registers and then returns. Note that _start_trap must be 4-byte aligned */
/* EXTERN(_default_start_trap);
PROVIDE(_start_trap = _default_start_trap); */

/* Default interrupt setup entry point. If not _setup_interrupts symbol is provided, then
   _setup_interrupts maps to _default_setup_interrupts, which in direct mode sets the value
   of the xtvec register to _start_trap and, in vectored mode, sets its value to
   _vector_table and enables vectored mode. */
/* EXTERN(_default_setup_interrupts);
PROVIDE(_setup_interrupts = _default_setup_interrupts); */

/* Default main routine. If no hal_main symbol is provided, then hal_main maps to main, which
   is usually defined by final users via the #[riscv_rt::entry] attribute. Using hal_main
   instead of main directly allow HALs to inject code before jumping to user main. */
/* PROVIDE(hal_main = main); */

/* Default exception handler. By default, the exception handler is abort.
   Users can override this alias by defining the symbol themselves */
/* PROVIDE(ExceptionHandler = abort); */

/* Default interrupt handler. By default, the interrupt handler is abort.
   Users can override this alias by defining the symbol themselves */
/* PROVIDE(DefaultHandler = abort); */

/* Default interrupt trap entry point. When vectored trap mode is enabled,
   the riscv-rt crate provides an implementation of this function, which saves caller saved
   registers, calls the the DefaultHandler ISR, restores caller saved registers and returns.
   Note, however, that this provided implementation cannot be overwritten. We use PROVIDE
   to avoid compilation errors in direct mode, not to allow users to overwrite the symbol. */
/* PROVIDE(_start_DefaultHandler_trap = _start_trap); */

/* ${INCLUDE_LINKER_FILES} */

/* PROVIDE(_stext = ORIGIN(REGION_TEXT));
PROVIDE(_stack_start = ORIGIN(REGION_STACK) + LENGTH(REGION_STACK));
PROVIDE(_max_hart_id = 0);
PROVIDE(_hart_stack_size = SIZEOF(.stack) / (_max_hart_id + 1));
PROVIDE(_heap_size = 0); */

ENTRY(__riscv_init_mmu)

TRAMPOLINE_LMA = 0x82000000;

KERNEL_LMA = 0x83000000;
KERNEL_VMA = 0xffffffff83000000;
KERNEL_VA_OFFSET = KERNEL_VMA - KERNEL_LMA;

USER_LMA = 0x85000000;
USER_VMA = 0x45000000;
USER_VA_OFFSET = USER_VMA - USER_LMA;

MEMORY
{
    TRAMPOLINE_REGION : ORIGIN = TRAMPOLINE_LMA, LENGTH = 16M
    KERNEL_REGION : ORIGIN = KERNEL_VMA, LENGTH = 32M
    USER_REGION   : ORIGIN = USER_VMA, LENGTH = 48M
}


PAGE_SIZE = 4096;

USER_HEAP_PAGES = 1;
USER_STACK_PAGES = 128;

SECTIONS
{

  .trampoline ALIGN(PAGE_SIZE) :
  {
    KEEP(*(.trampoline.text));

    . = ALIGN(PAGE_SIZE);
    KEEP(*(.trampoline.root_table));
  } > TRAMPOLINE_REGION

  __kernel_va_offset = KERNEL_VA_OFFSET;
  __user_va_offset = USER_VA_OFFSET;





  .user.text ALIGN(PAGE_SIZE) : AT(ADDR(.user.text) - USER_VA_OFFSET)
  {
    __sutext = .;
    /* *(.text.abort); */
    *libuser_programs.a:(.text .text.*);

    . = ALIGN(4);
    __eutext = .;
  } > USER_REGION

  .user.rodata ALIGN(PAGE_SIZE) : AT(ADDR(.user.rodata) - USER_VA_OFFSET)
  {
     . = ALIGN(4);
    __surodata = .;

    /* *(.srodata .srodata.*); */
    *libuser_programs.a:(.rodata .rodata.*);

    . = ALIGN(8);
    /* KEEP(*(.rodata.kernel_layout)); */

    /* 8-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(8);
    __eurodata = .;
  } > USER_REGION

  .user.data ALIGN(PAGE_SIZE) : AT(ADDR(.user.data) - USER_VA_OFFSET)
  {
    . = ALIGN(8);
    __sudata = .;

    /* Must be called __global_pointer$ for linker relaxations to work. */
    /* PROVIDE(__global_pointer$ = . + 0x800); */
    /* *(.sdata .sdata.* .sdata2 .sdata2.*); */
    *libuser_programs.a:(.data .data.*);

    . = ALIGN(8);
    __eudata = .;
  } > USER_REGION


  /* LMA of .data */
  /* __suidata = LOADADDR(.data); */

  .user.bss ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.user.bss) - USER_VA_OFFSET)
  {
    . = ALIGN(8);
    __subss = .;

    *libuser_programs.a:(/* .sbss .sbss.* */ .bss .bss.*);
    __eubss = .;
  } > USER_REGION

  /* Uninitialized data segment. In contrast with .bss, .uninit is not initialized to zero by
   * the runtime, and might contain residual data from previous executions or random values
   * if not explicitly initialized. While .bss and .uninit are different sections, they are
   * both allocated at REGION_BSS, as their purpose is similar. */
  .user.uninit ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.user.uninit) - USER_VA_OFFSET)
  {
    . = ALIGN(8);
    __suuninit = .;
    *libuser_programs.a:(.uninit);
    . = ALIGN(8);
    __euuninit = .;
  } > USER_REGION

  /* fictitious region that represents the memory available for the heap */
  .user.heap ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.user.heap) - USER_VA_OFFSET)
  {
    __suheap = .;
    /* . += _heap_size; */
    . += PAGE_SIZE * USER_HEAP_PAGES;
    . = ALIGN(4);
    __euheap = .;
  } > USER_REGION

  /* fictitious region that represents the memory available for the stack */
  .user.stack ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.user.stack) - USER_VA_OFFSET)
  {
    /* Page for stack protection */
    . = ALIGN(PAGE_SIZE);
    . += PAGE_SIZE * 10;
    __eustack = .;
    . += PAGE_SIZE * USER_STACK_PAGES;
    /* . = ABSOLUTE(_stack_start); */
    __sustack = .;
  } > USER_REGION







  .text ALIGN(PAGE_SIZE) : AT(ADDR(.text) - KERNEL_VA_OFFSET)
  {
    __stext = .;

    /* Put reset handler first in .text section so it ends up as the entry */
    /* point of the program. */
    KEEP(*(EXCLUDE_FILE(*libuser_programs.a) .init));
    
    . = ALIGN(4);
    
    /* Probably not needed */
    KEEP(*(EXCLUDE_FILE(*libuser_programs.a) .trap.vector));   /* for _trap_vector (vectored mode only) */
    KEEP(*(EXCLUDE_FILE(*libuser_programs.a) .trap.start));    /* for _start_trap routine */
    KEEP(*(EXCLUDE_FILE(*libuser_programs.a) .trap.start.*));  /* for _start_INTERRUPT_trap routines (vectored mode only) */
    KEEP(*(EXCLUDE_FILE(*libuser_programs.a) .trap.continue)); /* for _continue_trap routine (vectored mode only) */
    KEEP(*(EXCLUDE_FILE(*libuser_programs.a) .trap.rust));     /* for _start_trap_rust Rust function */
    KEEP(*(EXCLUDE_FILE(*libuser_programs.a) .trap .trap.*));  /* Other .trap symbols at the end */

    /* *(EXCLUDE_FILE(*libuser_programs.a) .text.abort); */
    *(.text .text.*);

    . = ALIGN(4);
    __etext = .;
  } > KERNEL_REGION

  .rodata ALIGN(PAGE_SIZE) : AT(ADDR(.rodata) - KERNEL_VA_OFFSET)
  {
     . = ALIGN(4);
    __srodata = .;

    /* *(EXCLUDE_FILE(*libuser_programs.a) .srodata .srodata.*); */
    *(EXCLUDE_FILE(*libuser_programs.a) .rodata .rodata.*);

    . = ALIGN(8);
    /* KEEP(*(.rodata.kernel_layout)); */

    /* 8-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(8);
    __erodata = .;
  } > KERNEL_REGION

  .data ALIGN(PAGE_SIZE) : AT(ADDR(.data) - KERNEL_VA_OFFSET)
  {
    . = ALIGN(8);
    __sdata = .;

    /* Must be called __global_pointer$ for linker relaxations to work. */
    PROVIDE(__global_pointer$ = . + 0x800);
    /* *(EXCLUDE_FILE(*libuser_programs.a) .sdata .sdata.* .sdata2 .sdata2.*); */
    *(.data .data.*);

    . = ALIGN(8);
    __edata = .;
  } > KERNEL_REGION


  /* LMA of .data */
  /* __sidata = LOADADDR(.data); */

  .bss ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.bss) - KERNEL_VA_OFFSET)
  {
    . = ALIGN(8);
    __sbss = .;

    *(EXCLUDE_FILE(*libuser_programs.a) /* .sbss .sbss.* */ .bss .bss.*);
    __ebss = .;
  } > KERNEL_REGION

  /* Uninitialized data segment. In contrast with .bss, .uninit is not initialized to zero by
   * the runtime, and might contain residual data from previous executions or random values
   * if not explicitly initialized. While .bss and .uninit are different sections, they are
   * both allocated at REGION_BSS, as their purpose is similar. */
  .uninit ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.uninit) - KERNEL_VA_OFFSET)
  {
    . = ALIGN(8);
    __suninit = .;
    *(EXCLUDE_FILE(*libuser_programs.a) .uninit);
    . = ALIGN(8);
    __euninit = .;
  } > KERNEL_REGION

  /* fictitious region that represents the memory available for the heap */
  .heap ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.heap) - KERNEL_VA_OFFSET)
  {
    __sheap = .;
    KEEP(*(.heap));
    . = ALIGN(8);
    __eheap = .;
  } > KERNEL_REGION

  /* fictitious region that represents the memory available for page table pool */
  .page_table_pool ALIGN(PAGE_SIZE) : AT(ADDR(.page_table_pool) - KERNEL_VA_OFFSET)
  {
    __spage_table_pool = .;
    KEEP(*(.page_table_pool));
    __epage_table_pool = .;
    __s_temp_mmu_table = .;
    . += PAGE_SIZE * 32;
    __e_temp_mmu_table = .;
    __s_temp_kernel_stacks = .;
    . += PAGE_SIZE * 512;
    __e_temp_kernel_stacks = .;
  } > KERNEL_REGION

  /* fictitious region that represents the memory available for the stack */
  .stack ALIGN(PAGE_SIZE) (NOLOAD) : AT(ADDR(.stack) - KERNEL_VA_OFFSET)
  {
    /* Page for stack protection */
    __sstack_protector = .;
    . += PAGE_SIZE;
    __estack_protector = .;
    __estack = .;
    KEEP(*(.stack));
    . = ALIGN(16);
    __sstack = .;
  } > KERNEL_REGION






  





  /* fake output .got section */
  /* Dynamic relocations are unsupported. This section is only used to detect
     relocatable code in the input files and raise an error if relocatable code
     is found */
  .got (INFO) :
  {
    KEEP(*(.got .got.*));
  } > KERNEL_REGION
}

/* Do not exceed this mark in the error messages above                                    | */
/* ASSERT(ORIGIN(REGION_TEXT) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_TEXT must be 4-byte aligned");

ASSERT(ORIGIN(REGION_RODATA) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_RODATA must be 4-byte aligned");

ASSERT(ORIGIN(REGION_DATA) % 8 == 0, "
ERROR(riscv-rt): the start of the REGION_DATA must be 8-byte aligned");

ASSERT(ORIGIN(REGION_HEAP) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_HEAP must be 4-byte aligned");

ASSERT(ORIGIN(REGION_STACK) % 4 == 0, "
ERROR(riscv-rt): the start of the REGION_STACK must be 4-byte aligned"); */

/* Add assert _start_trap */

/* ASSERT(_stext % 4 == 0, "
ERROR(riscv-rt): `_stext` must be 4-byte aligned"); */

ASSERT(__sdata % 8 == 0 && __edata % 8 == 0, "
BUG(riscv-rt): .data is not 8-byte aligned");

/* ASSERT(__sidata % 8 == 0, "
BUG(riscv-rt): the LMA of .data is not 8-byte aligned"); */

ASSERT(__sbss % 8 == 0 /* && __ebss % 8  == 0 */, "
BUG(riscv-rt): .bss is not 8-byte aligned");

ASSERT(__sheap % 4 == 0, "
BUG(riscv-rt): start of .heap is not 4-byte aligned");

ASSERT(KERNEL_VA_OFFSET % 1073741824 == 0, "
BUG(riscv-rt): KERNEL_VA_OFFSET is not gigabyte-aligned");

/* ASSERT(_pre_init_trap % 4 == 0, "
BUG(riscv-rt): _pre_init_trap is not 4-byte aligned"); */

ASSERT(_start_trap % 4 == 0, "
BUG(riscv-rt): _start_trap is not 4-byte aligned");

/* ASSERT(_stext + SIZEOF(.text) < ORIGIN(REGION_TEXT) + LENGTH(REGION_TEXT), "
ERROR(riscv-rt): The .text section must be placed inside the REGION_TEXT region.
Set _stext to an address smaller than 'ORIGIN(REGION_TEXT) + LENGTH(REGION_TEXT)'"); */

/* ASSERT(SIZEOF(.stack) >= (_max_hart_id + 1) * _hart_stack_size, "
ERROR(riscv-rt): .stack section is too small for allocating stacks for all the harts.
Consider changing `_max_hart_id` or `_hart_stack_size`."); */

/* # Other checks */
ASSERT(SIZEOF(.got) == 0, "
ERROR(riscv-rt): .got section detected in the input files. Dynamic relocations are not
supported. If you are linking to C code compiled using the `cc` crate then modify your
build script to compile the C code _without_ the -fPIC flag. See the documentation of
the `cc::Build.pic` method for details.");

/* Do not exceed this mark in the error messages above                                    | */
