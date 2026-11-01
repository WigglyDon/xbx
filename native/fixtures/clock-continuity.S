/* SPDX-License-Identifier: MIT
 * Original finite public clock/timer contract. UART and generated XBE envelope
 * are retained xbx fixture code. No title content, instruction or memory input.
 */
.ifndef XBX_TSC_CONTRACT
.set XBX_TSC_CONTRACT, 0
.endif
.text
.globl _start
_start:
    cld
    call serial_init
    movl $boot,%esi
    call text
    call *thunks+4               /* KeQueryPerformanceFrequency: EDX:EAX */
    testl %edx,%edx
    jnz fail
    cmpl $3375000,%eax
    jne fail
again:
    incl cycle
    movl $begin,%esi
    call text
    call *thunks+12              /* KeQueryInterruptTime, 100 ns */
    movl %eax,interrupt_start
    movl thunks+16,%eax          /* KeTickCount data */
    movl (%eax),%eax
    movl %eax,tick_start
    call *thunks                 /* KeQueryPerformanceCounter */
    movl %eax,pm_start
.if XBX_TSC_CONTRACT
    rdtsc
    movl %eax,tsc_start
    movl %edx,tsc_start+4
.endif
1:  call *thunks
    subl pm_start,%eax           /* one second, wrap-safe low-word delta */
    cmpl $3375000,%eax
    jb 1b
.if XBX_TSC_CONTRACT
    rdtsc
    subl tsc_start,%eax
    sbbl tsc_start+4,%edx
    movl %eax,tsc_delta
    movl %edx,tsc_delta+4
.endif
    call *thunks+12
    subl interrupt_start,%eax
    movl %eax,interrupt_delta
    movl thunks+16,%eax
    movl (%eax),%eax
    subl tick_start,%eax
    movl %eax,tick_delta
    movl $clocks,%esi
    call text
    movl interrupt_delta,%eax
    call hex
    movl $ticks,%esi
    call text
    movl tick_delta,%eax
    call hex
    movl $newline,%esi
    call text
.if XBX_TSC_CONTRACT
    movl $tsc_text,%esi
    call text
    movl tsc_delta+4,%eax
    call hex
    movl tsc_delta,%eax
    call hex
    movl $newline,%esi
    call text
    /* Retained hw/i386/x86-cpu.c supplies 733333333 ticks per virtual second.
     * The loose bounds permit ordinary host scheduling, not a different rate.
     */
    cmpl $0,tsc_delta+4
    jne fail
    cmpl $700000000,tsc_delta
    jb fail
    cmpl $770000000,tsc_delta
    ja fail
.endif
    cmpl $500,tick_delta
    jb fail
    cmpl $2000,tick_delta
    ja fail
    cmpl $5000000,interrupt_delta
    jb fail
    cmpl $20000000,interrupt_delta
    ja fail
    call *thunks
    movl %eax,pm_start
    movl $delay_begin,%esi
    call text
    pushl $interval
    pushl $0                     /* non-alertable */
    pushl $0                     /* KernelMode */
    call *thunks+8               /* KeDelayExecutionThread, one second */
    testl %eax,%eax
    jnz fail
    call *thunks
    subl pm_start,%eax
    movl %eax,pm_delta
    movl $delay_done,%esi
    call text
    movl pm_delta,%eax
    call hex
    movl $newline,%esi
    call text
    cmpl $1687500,pm_delta
    jb fail
    cmpl $6750000,pm_delta
    ja fail
    movl $passed,%esi
    call text
    movl cycle,%eax
    call hex
    movl $newline,%esi
    call text
    /* Return to this ordinary guest UART command loop after a checkpoint. */
2:  movw $0x3fd,%dx
    inb %dx,%al
    testb $1,%al
    jnz 3f
    sti
    hlt
    jmp 2b
3:  movw $0x3f8,%dx
    inb %dx,%al
    cmpb $'c',%al
    je again
    jmp 2b
fail:
    movl $failed,%esi
    call text
4:  sti
    hlt
    jmp 4b
.data
.globl thunks
thunks:
    .long 0x8000007e             /*126 KeQueryPerformanceCounter */
    .long 0x8000007f             /*127 KeQueryPerformanceFrequency */
    .long 0x80000063             /* 99 KeDelayExecutionThread */
    .long 0x8000007d             /*125 KeQueryInterruptTime */
    .long 0x8000009c             /*156 KeTickCount data */
    .long 0
interval: .quad -10000000
cycle: .long 0
pm_start: .long 0
pm_delta: .long 0
interrupt_start: .long 0
interrupt_delta: .long 0
tick_start: .long 0
tick_delta: .long 0
.if XBX_TSC_CONTRACT
tsc_start: .quad 0
tsc_delta: .quad 0
tsc_text: .asciz "XBX CLOCK tsc_delta="
.endif
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
boot: .asciz "XBX CLOCK BOOT\n"
begin: .asciz "XBX CLOCK BEGIN\n"
clocks: .asciz "XBX CLOCK interrupt_delta="
ticks: .asciz " tick_delta="
delay_begin: .asciz "XBX CLOCK DELAY BEGIN\n"
delay_done: .asciz "XBX CLOCK DELAY DONE pm_delta="
passed: .asciz "XBX CLOCK PASS cycle="
failed: .asciz "XBX CLOCK FAIL\n"
newline: .asciz "\n"
