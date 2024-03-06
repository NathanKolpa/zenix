# Booting

## Pre-kernel

Zenix tries to optimize boot performance based on the Qemu emulator. Generally, the most performant method of booting is though the `-kernel` flag. In this case, Qemu will try to find a [Multiboot](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html) in a elf32 executable. This executable in Zenix is called the "Pre-kernel".

Zenix primarily targets 64-bit and multiboot can only boot into 32-bit (even on 64-bit computers). This task of [switching from Protected to Long Mode](https://wiki.osdev.org/Setting_Up_Long_Mode) is done by the Pre-kernel. During this switch, the Pre-kernel has to setup the inital [page tables](https://os.phil-opp.com/paging-introduction/). The setup of these page tables requires that pyhsical memory is allocated. For the Pre-kernel, a simple [Bump Allocator](https://os.phil-opp.com/allocator-designs/) suffices. Because multiple parts of the Kernel require pyhsical memory access, the full physical memory is mapped with an offset of 60 TiB. Futhermore, the Pre-kernel iself and "Bump memory" is [identity mapped](https://wiki.osdev.org/Identity_Paging).

The actual Zenix Kernel is not part of the Pre-kernel, they are seperate executables. Another benifit of having 2 seperate executables, is that we can skip linking the two files during compilation. Therefore, speeding up build times. Linking these two executables is also not a trivial task, because they target 2 different architectures. Qemu loads the Kernel into memory with the `-inird` (inital ramdisk) flag. The Pre-kernel can then identify where the Kernel is placed in memory though the use of Multiboot's module feature.

The kernel mappings are backed by the physical memory of the Multiboot module, saving the overhead of copying the data of the module to the Kernel's mappings. There is a exceptions to this rule. Some sections are not present within the Kernel's elf file, but are still required to be mapped in memory. These sections are most likely static uninitialized data (commonly called the *.bss section*). This means that backing the memory mappings with the Mulitboot module is not possible. The parts of these sections located in the bump memory instead.

The last step of the Pre-kernel is place information that can only be optained in at this stage of the boot process into the bump memory. So that, the Kernel can access this information throughout it's lifetime. Multiboot information is lost after paging is enabled.

Finally, the Pre-kernel can call `kernel_main` and the actual Kernel starts running.

## Kernel

When the Pre kernel is done, there is a lot of memory left within bump memory. The kernel makes use of this by repurposing whatever is left as the kernel heap. This marks the final stage of the bump memory.

![Bump Memory](./diagrams/bump-memory/bm.svg)
