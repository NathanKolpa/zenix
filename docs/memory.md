# Memory

Required reading:

- [Paging Introduction](https://os.phil-opp.com/paging-introduction/) 
- [Booting](./boot.md)

## Physical Memory

In order to manage virtual memory, the Kernel needs to manage physical memory first. The Zenix kernel uses the same underlying algoritm as the [Linux Kernel](https://www.kernel.org/doc/gorman/html/understand/understand009.html). The reason for using [**Buddy Allocation**](https://www.youtube.com/watch?v=DRAHRJEAEso) is because physical memory has one big diffrence compared to traditional memory. Which is the fact that, physical memory only gets allocated in blocks of 4KiB 2MiB and 1GiB. This midigates the main downside of the Buddy Allocator, which is that each allocation has to be a power of 2. How convenient!

The memory that is available is not represented as a single flat line, instead the Pre-Kernel gives a list of usable **regions**. Each region gets its own "heap", this is called a **Zone** ([the same name that linux uses](https://litux.nl/mirror/kerneldevelopment/0672327201/ch11lev1sec2.html)). 

Each zone keeps track of a list of so called **Levels** where each index of a level corresponds to the order of magnitude of which that level manages. The order of magitude here refers to the size of the allocation. Say, 4KiB is requested, the order would be 12. Each level consists of a (double-linked) [freelist](https://en.wikipedia.org/wiki/Free_list) and a bitmap. The freelist here keeps track oblocks of free memory. The bitmap marks each block as used or free. This is usefull for navigating the freelist, because if you know when a block is free, then you don't have to walk the entire list in order to get the pointer. Instead its possible to calculate the address and derefrence the memory as a node. Which keeps the complexity for the algorithm to `O(log(n))`.

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

## Virtual Memory


