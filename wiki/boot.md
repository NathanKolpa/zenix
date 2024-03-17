# The boot process

This chapter describes the boot process from the moment the first line is executed, to the Kernel switching to userspace for the first time. The text contains links to code files which are ment to help clarify the code structure.

| ![Boot Package Diagram](./diagrams/boot-package/bp.svg) |
|:--:|
| *Relevant crates and their dependencies* |

## Pre-kernel

Zenix tries to optimize boot performance based on the Qemu emulator. Generally, the most performant method of booting is though the `-kernel` flag. In this case, Qemu will try to find a Multiboot header[^1] in a elf32 executable. This executable in Zenix is called the *Pre-kernel*.

| ![Pre-kernel sequence](./diagrams/pre-kernel-flow/seq.svg) |
|:--:|
| *Sequence diagram of the Pre-kernel* |

The first few instructions are dedicated to [setting up the stack](../pre-kernel/src/boot.s). This task can only be done in assembly because Rust makes the assumption that a stack is always there.

### Initial mappings

Zenix primarily targets 64-bit and multiboot can only boot into 32-bit (even on 64-bit computers). This task of [switching from Protected to Long Mode](../pre-kernel/src/long_mode.rs)[^2] is done by the Pre-kernel. Before this switch, the Pre-kernel has to setup [the inital page tables](../pre-kernel/src/paging.rs)[^3]. The setup of these page tables requires that pyhsical memory is allocated. For the Pre-kernel, a simple [`BumpAllocator`](../pre-kernel/src/bump_memory.rs)[^4] suffices because memory never has to be deallocated. Multiple parts of the Kernel require arbitrary pyhsical memory access, the full physical memory is mapped with an offset of 60 TiB[^3]. Other parts of the memory are identity mapped[^5], these include:

* The Pre-kernel code.
* The Bump Memory.
* The Pre-kernel/Kernel stack.
* The Root System Descriptor Table.
* The VGA buffer.
* The Local APIC.

### Mapping the Kernel

The actual Zenix Kernel is not part of the Pre-kernel, they are seperate executables. Another benifit of having 2 seperate executables, is that we can skip linking the two files during compilation. Therefore, speeding up build times. Linking these two executables is also not a trivial task, because they target 2 different architectures. Qemu loads the Kernel into memory with the `-inird` (inital ramdisk) flag. The Pre-kernel can then identify where the Kernel is placed in memory though the use of Multiboot's [module feature](../pre-kernel/src/multiboot.rs).

The kernel mappings are backed by the physical memory of the Multiboot module, saving the overhead of copying the data of the module to the Kernel's mappings. There is a exceptions to this rule. Some sections are not present within the Kernel's elf file, but are still required to be mapped in memory. These sections are most likely static uninitialized data (commonly called the *.bss section*). This means that backing the memory mappings with the Mulitboot module is not possible. The parts of these sections located in the bump memory instead.

### Boot information

Another task of the Pre-kernel is to save information that can would be lost after switching to long mode/calling to the kernel. This (boot) information [gets stored in the bump memory](../pre-kernel/src/boot_info.rs) and gets passed to the kernel.

After the Pre-kernel is done, there is still a lot of memory left within bump memory. The kernel can make use of this by repurposing whatever is left as backing for the heap. This marks the final stage of the Bump Memory. The Kernel can also expliot the fact that the Pre-kernel is right ajacent to the Bump Memory. The memory used by the Pre-kernel can be trivially relcaimed by simply placing it in the heap. The kernel is not aware of this this however, the merging is actually done by the Pre-kernel when setting up the boot information.

| ![Bump Memory](./diagrams/bump-memory/bm.svg) |
|:--:|
| *Bump memory in its final stage* |

Finally, the Pre-kernel can switch to long mode and enter the kernel by calling [`kernel_main`](../kernel/src/main.rs).

[^1]: Gnu: [Multiboot Specification version 0.6.96](https://www.gnu.org/software/grub/manual/multiboot/multiboot.html)
[^2]: Osdev: [Setting up long mode](https://wiki.osdev.org/Setting_Up_Long_Mode)
[^3]: Phil Opp: [Paging Introduction](https://os.phil-opp.com/paging-introduction/)
[^4]: Phil Opp: [Bump Allocator](https://os.phil-opp.com/allocator-designs/)
[^5]: Osdev: [Identity Paging](https://wiki.osdev.org/Identity_Paging)

## Kernel heap

Now begins the first step of the [kernel initialization](../kernel/src/init.rs).

TODO: write about the heap allocation algorithm, [For readers](https://os.phil-opp.com/heap-allocation/).

## Physical Memory

In order to manage virtual memory, the Kernel needs to manage physical memory first using the [`FrameAllocator`](../kernel/src/memory/alloc/frame_alloc.rs). The Zenix kernel uses the Buddy Allocation System[^7] which is same algoritm as the Linux Kernel[^6] uses. The reason for using Buddy Allocation is because physical memory has one big diffrence compared to traditional memory. Which is the fact that, physical memory only gets allocated in blocks of 4KiB, 2MiB and 1GiB. This midigates the main downside of the Buddy Allocator, which is that each allocation has to be a power of 2. How convenient! Buddy allocation has a very good time complexity of `O(Log(N))`.

The memory that is available for the Kernel is not represented as a single flat line, instead, the Pre-Kernel gives a list of usable **regions**. Each region gets its own "heap", this is called a [`Zone`](../kernel/src/memory/alloc/frame_alloc/zone.rs) (the same name that linux uses[^8]).

Each zone keeps track of an array of a so called [`Level`](../kernel/src/memory/alloc/frame_alloc/level.rs) where each index of a level corresponds to the order of magnitude of which that level manages. The order of magitude here refers to the size of the allocation. Say, 4KiB is requested, the order would be 12. Each level consists of a (double-linked) freelist[^9] and a [`Bitmap`](../libraries/essentials/src/bitmap.rs). The freelist here keeps track of blocks of free memory. The bitmap marks each block as used or free. This is usefull for navigating the freelist, because when a block is free, then you don't have to walk the entire list in order to get the node. Instead its possible to calculate the address and derefrence the memory as a node. Which keeps the complexity for the algorithm to `O(log(n))`.

[^6]: Kernel archives: [Chapter 6 Physical Page Allocation](https://www.kernel.org/doc/gorman/html/understand/understand009.html)
[^7]: Gabriel Parmer: [GWU OS: Memory Allocation - Slab and Buddy Allocators](https://www.youtube.com/watch?v=DRAHRJEAEso)
[^8]: Robert Love: [Zones](https://litux.nl/mirror/kerneldevelopment/0672327201/ch11lev1sec2.html)
[^9]: Wikipedia: [Free list](https://en.wikipedia.org/wiki/Free_list)

### Initialization

Unfortunately, a region is almost never perfectly a power of two. A naive solution to this problem would be to round down to the previous power of two. Lets say we have 119 Mib (an actual region size from qemu), then rounding down to 64 Mib would cost us almost half our memory!

This problem can be solved by allowing incomplete blocks: first the region size is rounded up to the next nearest power of two so that 119 Mib would become 128 MiB. Then, for each level we take a piece of usable memory and make it available by adding it to the level's free list. And avoid to coalescing incomplete blocks when deallocating, each link listed block's buddy is marked as allocated. If a piece memory doesnt' fit in a level, we mark it as allocated and move on. We repeat the above steps until there is no more usable memory left.

An example of how 344.0 KiB looks in a zone when initialized:
```txt
512 KiB:          *
                .` `.
               /     \
256 KiB:      @       *
            /  \     / \
128 KiB:   .    .   *   .
          /|   /|  /|  /|
64 KiB : . .  . . @ * . .

* = Marked as allocated in the level's bitmap.
@ = Added in the the level's free list.
. = Neither in the level's bitmap nor the free list.
```

### Allocation

First, the size of the request is rounded up to the next nearest power of two. For example, if a request for 5000 bytes is made, the allocation is rounded up to 8192 bytes. The corresponding level is then determined based on the new size.

If it's possible to pop a block of memory from the level's freelist, it is marked as used in the bitmap, and the request can be satisfied using (the address of) this block. If there is no available block of memory in the freelist, a recursive attempt is made to allocate a block in the level above. When the level above returns a block (exactly twice the size needed), the block is split in half. The first half is added to the freelist, and the second half is marked as used in the bitmap. Subsequently, the request is fulfilled using the second half.

### Deallocation

Before its possible to start the de-allocation process we need to figure out the size of the block of memory we want to free.
This can be done by checking for each level (from small to large) if the block is marked as used.
If the block is used, then the level's size is our de-allocation size.

Using this size, we get the corresponding level. In the level's bitmap, we then mark the block of memory as unused. If the the block's buddy is also unused, we can coalesce the two blocks so in the future larger requests can be satisfied. We achieve this by removing the buddy block from the freelist. Then we recursively deallocate for the next level. If it is not possible to coalesce a block we add it to the level's freelist.


## Virtual Memory

## Interrupts

Understanding how interrupts work and are handled in the kernel is important because it dictates what codes gets to run at what time. In this section the low level details of how interrupts work are covered.

### Legacy

One of the central hardware features that drives the kernel are interrupts. In simple terms, an interrupt is a function/callback that can be called by the hardware. But who calls these magical functions? You may guess that this is done by the processor chip itself, but its actually a seperate chip called the [`PIC`](../libraries/x86_64/src/device/pic_8259.rs)[^10]. No, actually there are two daisy chained together... No actually, you have to disable both of them and use ANTOHER chip called the [`APIC`](../libraries/x86_64/src/device/pic_8259.rs)[^11]. Like with many things of the x86_64 architecture, there are a lot of parts that merely exist for legacy reasons.

Because it's not guaranteed that the APIC chip is available on a CPU, Zenix supports both of them. By using the [`cpuid`](../libraries/x86_64/src/cpuid.rs) instruction its possible to detect the presence of this chip. Unlike it's predecessor, the way the kernel interacts with the APIC is rather complicated.

[^10]: Osdev: [8259 PIC](https://wiki.osdev.org/8259_PIC)
[^11]: Osdev: [APIC](https://wiki.osdev.org/APIC)

### The APIC

## Concurrency Safety

One important detail to understand is that these interrupts can happen at any time. Even in the middle of other code. Not accounting for this quirk can lead to wildy undefined behaviour. Lucily, Rust helps us prevent these issues at compile time. You may recall Rust being thread safe, but threads unsafety is caused by interrupts. Therefore, even when only one thread is running. Its still possible to run into issues such as race conditions when being careless.

The most simple way to synchronise between threads is by using a [`Spinlock`](../libraries/essentials/src/spin/lock.rs). This lock is like a Mutex, but it's underlying waiting mechanism is by running a infinite loop untill the lock is unlocked. There is one catch however: **deadlocks**. Say, a thread locks a resource. During this lock an interrupt is triggered. In the **interrupt handler** the handler function tries to lock the same resource. What happens now? Simple, nothing. And nothing until the end of time! Technically, a deadlock is not undefined behaviour but merely undesired behaviour. This means debugging is not absolutly deadfull.

In userspace or with real-time operating systems, deadlocks are *mostly* not a concern because a Mutex switches the current thread to another thread. Because threads do not wait, the deadlock does not occour. So why use these Spinlocks? Well, the kernel is trying to implement the fancy Mutexes, but the environment constrains the kernel because the Scheduler is not initialized/active *yet*. If you're familliar with Linux source code, you'll see how often these locks are used.

The way to solve deadlocks is to disable interrupts before locking the resouce. By wrapping the SpinLock with an [`InterruptGuard`](../kernel/src/utils/interrupt_guard.rs) helper, interrupts are automatically managed correctly.

## Multiprocessing


## VFS
## Syscalls
## Scheduler
