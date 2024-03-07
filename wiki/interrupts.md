# Interrupts

Understanding how interrupts work and are handled in the kernel is important because it dictates what codes gets to run at what time. This chapter covers the low level details of how interrupts work.

## Legacy

One of the central cpu feature that drives the kernel are interrupts. In simple terms, an interrupt is a function/callback that can be called by the hardware. But who calls these magical functions? You may guess that this is done by the processor chip itself, but its actually a [seperate chip](https://wiki.osdev.org/8259_PIC). No, actually there are two daisy chained together... No actually, you have to disable both of them and use ANTOHER chip called the [**APIC**](https://wiki.osdev.org/APIC). Like with many things of the x86_64 architecture, there are a lot of parts that merely exist for legacy reasons.

Because it's not guaranteed that the APIC chip is available on a CPU, Zenix supports both of them. By using the `cpuid` instruction its possible to detect the presence of this chip. Unlike it's predecessor, the way the kernel interacts with the APIC is rather complicated.

## The APIC

## Safety

One important detail to understand is that these interrupts can happen at any time. Even in the middle of other code. Not accounting for this quirk can lead to wildy undefined behaviour. Lucily, Rust helps us prevent these issues at compile time. You may recall Rust being thread safe, but threads unsafety is caused by interrupts. Therefore, even when only one thread is running. Its still possible to run into issues such as race conditions when being careless.

The most simple way to synchronise between threads is by using a **Spinlock**. This lock is like a Mutex, but it's waiting mechanism is by running a infinite loop untill the lock is unlocked. There is one catch however: **deadlocks**. Say, a thread locks a resource. During this lock an interrupt is triggered. In the **interrupt handler** the handler function tries to lock the same resource. What happens now? Simple, nothing. And nothing until the end of time! Technically, a deadlock is not undefined behaviour but merely undesired behaviour. This means debugging is not absolutly deadfull.

In userspace or with real-time operating systems, deadlocks are *mostly* not a concern because a Mutex switches the current thread to another thread. Because threads do not wait, the deadlock does not occour. So why use these Spinlocks? Well, the kernel is trying to implement the fancy Mutexes, but the environment constrains the kernel because the Scheduler is not initialized/active *yet*. If you're familliar with Linux source code, you'll see how often these locks are used.

The way to solve deadlocks is to disable interrupts before locking the resouce.
