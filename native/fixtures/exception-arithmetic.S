/* SPDX-License-Identifier: MIT
 * Original precise-exception arithmetic-state test; no commercial code.
 * ABI: retained public OXOS ke/except.cpp CONTEXT (Ecx 0x218, Eip 0x224,
 * EFlags 0x22c); exception registration follows original nullcall.S.
 * ISA: Intel SDM vol2A CMP/SUB and MOV, status bits CF/PF/AF/ZF/SF/OF.
 * https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2a-manual.pdf
 * A later ADD overwrites flags, but the preceding fault must expose CMP state.
 * No PUSHF/LAHF between CMP and the load: an explicit flag consumer would
 * conceal the liveness boundary being tested.
 */
.set STATUS_MASK, 0x8d5
.text
.globl _start
_start:
    cld
    call npx_serial_init
    pushl $handler
    pushl %fs:0
    movl %esp,%fs:0
    movl $1,stage
    movl $0x95,expected_flags
    movl $local_fault,expected_pc
    movl $local_return,resume_pc
    xorl %edx,%edx
    movl $1,%ecx
    cmpl $2,%ecx
local_fault:
    movl (%edx),%ecx
    addl $7,%ecx
    jmp unexpected_success
local_return:
    pushfl
    popl %eax
    andl $STATUS_MASK,%eax
    cmpl expected_flags,%eax
    jne wrong_return_flags
    cmpl $1,%ecx
    jne wrong_return_register
    movl $0xcc000001,%ecx
    call npx_report
    movl $2,stage
    movl $0x44,expected_flags
    movl $branch_fault,expected_pc
    movl $branch_return,resume_pc
    xorl %edx,%edx
    movl $1,%ecx
    cmpl $1,%ecx
    jmp branch_fault
    .p2align 6
branch_fault:
    movl (%edx),%ecx
    addl $7,%ecx
    jmp unexpected_success
branch_return:
    pushfl
    popl %eax
    andl $STATUS_MASK,%eax
    cmpl expected_flags,%eax
    jne wrong_return_flags
    cmpl $1,%ecx
    jne wrong_return_register
    movl $0xcc000002,%ecx
    call npx_report
    popl %eax
    movl %eax,%fs:0
    addl $4,%esp
    movl $0x43435041,%ecx
    call npx_report
halt:
    sti
    hlt
    jmp halt
handler:
    movl 4(%esp),%eax
    cmpl $0xc0000005,(%eax)
    jne wrong_exception
    movl expected_pc,%ecx
    cmpl %ecx,12(%eax)
    jne wrong_exception
    movl 12(%esp),%edx
    cmpl %ecx,0x224(%edx)
    jne wrong_exception
    cmpl $1,0x218(%edx)
    jne wrong_context_register
    movl 0x22c(%edx),%eax
    andl $STATUS_MASK,%eax
    cmpl expected_flags,%eax
    jne wrong_context_flags
    movl resume_pc,%eax
    movl %eax,0x224(%edx)
    xorl %eax,%eax
    ret
unexpected_success:     movl $0xbad00001,%ecx; jmp fail
wrong_exception:        movl $0xbad00002,%ecx; jmp fail
wrong_context_flags:    movl $0xbad00003,%ecx; jmp fail
wrong_return_flags:     movl $0xbad00004,%ecx; jmp fail
wrong_context_register: movl $0xbad00005,%ecx; jmp fail
wrong_return_register:  movl $0xbad00006,%ecx
fail:
    call npx_report
    movl stage,%ecx
    call npx_report
    movl $failure_signal,%esi
    call npx_text
    jmp halt
.data
.balign 4
.globl thunks
thunks: .long 0
stage: .long 0
expected_flags: .long 0
expected_pc: .long 0
resume_pc: .long 0
failure_signal: .asciz "*** fatal error 98\n"
