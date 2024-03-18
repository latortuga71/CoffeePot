# CoffeePot



# TODO


* Implement 64bit Instructions
* Implement C extension
* Implement D extension
* Implement Mul Extension
* Better Virtual Memory 
* Load Static ELF's That Include C runtime (was missing C extension or could rebuild toolchain to support multilib?)
* Snapshot & Restart
* More Syscalls




# How to build for RV64I

* riscv64-unknown-linux-gnu-gcc -S basic.c
* Add two lines to basic.s 
```
.option norvc
.attribute arch, "rv64i"
```

* riscv64-unknown-linux-gnu-gcc -Wl,-Ttext=0x0 -nostdlib -march=rv64i -mabi=lp64 -o basic basic.s
* riscv64-unknown-linux-gnu-objcopy -O binary basic basic.bin

```
riscv64-unknown-linux-gnu-objdump basic -s -j .text -d
```
Running above should show something like this. 32 Bit long instructions instead of c extension 16 bit

```
0000000000000000 <main>:
   0:   fe010113                addi    sp,sp,-32
   4:   00813c23                sd      s0,24(sp)
   8:   02010413                addi    s0,sp,32
   c:   00050793                mv      a5,a0
  10:   feb43023                sd      a1,-32(s0)
  14:   fef42623                sw      a5,-20(s0)
  18:   00400793                li      a5,4
  1c:   00078513                mv      a0,a5
  20:   01813403                ld      s0,24(sp)
  24:   02010113                addi    sp,sp,32
  28:   00008067                ret
```
