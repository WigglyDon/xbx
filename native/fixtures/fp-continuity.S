/* SPDX-License-Identifier: MIT
 * Original live x87 stack, control word, XMM register, MXCSR and RAM-cookie
 * continuity test. Uses only original constants and the retained UART helper.
 */
.text
.globl _start
_start:
    cld
    call serial_init
    fninit
    fldcw control_word
    ldmxcsr mxcsr
    movdqu pattern,%xmm0
    fldl first
    fldl second
    movl $0x5afe9137,cookie
    movl $ready,%esi
    call text
command:
    movw $0x3fd,%dx
    inb %dx,%al
    testb $1,%al
    jnz read_command
    sti
    hlt
    jmp command
read_command:
    movw $0x3f8,%dx
    inb %dx,%al
    cmpb $'v',%al
    jne command
    cmpl $0x5afe9137,cookie
    jne fail
    movdqu %xmm0,observed_pattern
    movl $4,%ecx
    movl $pattern,%esi
    movl $observed_pattern,%edi
    repe cmpsl
    jne fail
    fld %st(0)
    fadd %st(2),%st(0)
    fstpl observed_sum
    cmpl $0,observed_sum
    jne fail
    cmpl $0x40130000,observed_sum+4 /* 3.5 + 1.25 = 4.75, exactly */
    jne fail
    fnstcw observed_cw
    cmpw $0x027f,observed_cw
    jne fail
    fnstsw %ax
    testw $0x3f,%ax
    jnz fail
    andw $0x3800,%ax
    cmpw $0x3000,%ax             /* two live entries; TOP=6 */
    jne fail
    stmxcsr observed_mxcsr
    cmpl $0x1f80,observed_mxcsr
    jne fail
    movl $passed,%esi
    call text
    jmp command
fail:
    movl $failed,%esi
    call text
1:  sti
    hlt
    jmp 1b
.data
.globl thunks
thunks: .long 0
control_word: .short 0x027f
observed_cw: .short 0
mxcsr: .long 0x1f80
observed_mxcsr: .long 0
cookie: .long 0
first: .quad 0x3ff4000000000000
second: .quad 0x400c000000000000
observed_sum: .quad 0
.balign 16
pattern: .long 0x13579bdf,0x2468ace0,0x3f800000,0xdead17a5
observed_pattern: .space 16
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
ready: .asciz "XBX FP_STATE READY\n"
passed: .asciz "XBX FP_STATE PASS\n"
failed: .asciz "XBX FP_STATE FAIL\n"
