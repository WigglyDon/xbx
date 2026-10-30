/* SPDX-License-Identifier: MIT
 * Original finite dispatch/SSE workload, no game code or data. UART routines
 * are generated from our retained startup-lifetime fixture. Separate code/data
 * pages avoid measuring self-modification. All arithmetic is exactly representable.
 */
.text
.globl _start
_start:
    cld
    call serial_init
    movl $dispatch_begin,%esi
    call text
    xorl %ebx,%ebx
    xorl %eax,%eax
    movl $4000000,%ecx
1:  movl targets(,%ebx,4),%edx
    call *%edx
    xorl $1,%ebx
    decl %ecx
    jnz 1b
    cmpl $DISPATCH_EXPECTED,%eax
    jne fail
    movl $dispatch_done,%esi
    call text
    movl $sse_exact_begin,%esi
    call text
    ldmxcsr mxcsr_exact
    call arithmetic
    stmxcsr status
    movl status,%eax
    andl $63,%eax
    jnz fail
    movl $sse_exact_done,%esi
    call text
    movl $sse_sticky_begin,%esi
    call text
    ldmxcsr mxcsr_sticky
    call arithmetic
    stmxcsr status
    movl status,%eax
    andl $63,%eax
    cmpl $32,%eax
    jne fail
    movl $sse_sticky_done,%esi
    call text
    movl $passed,%esi
    call text
2:  sti
    hlt
    jmp 2b
target_a:
    addl $3,%eax
    ret
target_b:
    xorl $0x5a5a5a5a,%eax
    ret
arithmetic:
    movaps quarter,%xmm0
    movaps half,%xmm1
    movaps eighth,%xmm2
    movl $4000000,%ecx
3:  mulps %xmm1,%xmm0
    addps %xmm2,%xmm0
    decl %ecx
    jnz 3b
    movaps %xmm0,result
    cmpl $0x3e800000,result
    jne fail
    cmpl $0x3e800000,result+4
    jne fail
    cmpl $0x3e800000,result+8
    jne fail
    cmpl $0x3e800000,result+12
    jne fail
    ret
fail:
    movl $failed,%esi
    call text
    jmp 2b
.data
.globl thunks
thunks: .long 0
targets: .long target_a,target_b
mxcsr_exact: .long 0x1f80
mxcsr_sticky: .long 0x1fa0
status: .long 0
.balign 16
quarter: .long 0x3e800000,0x3e800000,0x3e800000,0x3e800000
half: .long 0x3f000000,0x3f000000,0x3f000000,0x3f000000
eighth: .long 0x3e000000,0x3e000000,0x3e000000,0x3e000000
result: .space 16
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
dispatch_begin: .asciz "XBX CPU DISPATCH BEGIN\n"
dispatch_done: .asciz "XBX CPU DISPATCH DONE\n"
sse_exact_begin: .asciz "XBX CPU SSE EXACT BEGIN\n"
sse_exact_done: .asciz "XBX CPU SSE EXACT DONE\n"
sse_sticky_begin: .asciz "XBX CPU SSE STICKY BEGIN\n"
sse_sticky_done: .asciz "XBX CPU SSE STICKY DONE\n"
passed: .asciz "XBX CPU PERF PASS\n"
failed: .asciz "XBX CPU PERF FAIL\n"
.section .note.GNU-stack,"",@progbits
