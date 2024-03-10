.globl _start
.text
_start:
	/*
	ADDI sp, sp, -1024
	SB  x12, 0(x10) /* copy value at x13 into memory address at x10 with offset 0) 
	ADDI x10, sp, 8   get 8 bytes for stack space
	*/
	ADDI sp, sp, -32
	ADDI x15, sp, 0   /* get 8 bytes for stack space*/
	ADDI x14, x0, 67
	ADDI x13, x0, 0
	/* store bytes in memory */
	SB  x14, 0(x15) /* copy value at x12 into memory address at x11 with offset 0) */
	SB  x14, 1(x15) /* copy value at x12 into memory address at x11 with offset 1) */
	SB  x14, 2(x15) /* copy value at x12 into memory address at x11 with offset 2) */
	SB  x14, 3(x15) /* copy value at x12 into memory address at x12 with offset 3) */
	SB  x13, 4(x15) /* NULL
	/* write syscall */
	ADDI x17, x0, 64
	ADDI x10, x0, 1 /* stdout is 1 */
	ADDI x11, x15, 0 /* buffer address */
	ADDI x12, x0, 5 /* count should be 5 bytes */
	ecall 
	

	/* exit syscall */
	addi x17 ,x0, 93 /* a7 holds syscall numver */
	addi x10, x0, 99
	ecall


