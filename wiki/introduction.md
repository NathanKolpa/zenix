# Introduction

Zenix is a operating system that aims to improve the security by giving programs the minimum amount of permissions needed by default.
This document will give a brief overview on how the system achieves these goals.

## Security

The kernel manages permissions in a hieracical manner, meaning each child process inherits the permissions from the parent process.
When a parent process spawns a new child process, the parent gives a list of permisssions that the child will have.
The child's permissions may only be a subset of the parent's permissions.

A premission can be one of the following:
- Permission to read at a path.
- Permission to modify at a path.
- Permission to utilize the CPU for a amout of time (default: inherit)
- Permisison to utilize RAM for a amount given (default: inherit)
- Permission to execute certain syscalls (default: inherit)

A program can verify the existance of a permission though the `require` syscall.
