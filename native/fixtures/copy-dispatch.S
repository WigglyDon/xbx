/* SPDX-License-Identifier: MIT
 * Original bounded REP-copy / readonly indirect-dispatch control.
 * Synthetic bytes only. Rust owns construction and acceptance.
 */
.text
.globl _start
_start:
    cld
    call npx_serial_init
    xorl %ecx, %ecx
1:  movb %cl, source(%ecx)
    incl %ecx
    cmpl $80, %ecx
    jb 1b
    xorl %ebp, %ebp
2:  xorl %ebx, %ebx
3:  movl $destination, %edi
    movl $80, %ecx
    movb $0xa5, %al
    rep stosb
    leal destination+4(%ebp), %edi
    pushl %ebx
    pushl $source+1
    pushl %edi
    call copy_dispatch
    addl $12, %esp
    leal destination+4(%ebp), %edi
    xorl %ecx, %ecx
4:  cmpl %ebx, %ecx
    jae 5f
    leal 1(%ecx), %eax
    cmpb %al, (%edi,%ecx)
    jne failed
    incl %ecx
    jmp 4b
5:  cmpb $0xa5, -1(%edi)
    jne failed
    cmpb $0xa5, (%edi,%ebx)
    jne failed
    incl completed
    incl %ebx
    cmpl $64, %ebx
    jbe 3b
    incl %ebp
    cmpl $4, %ebp
    jb 2b
    movl completed, %ecx
    call npx_report
    movl $0x52455041, %ecx
    call npx_report
6:  sti
    hlt
    jmp 6b
failed:
    movl completed, %ecx
    call npx_report
    movl $0x52455046, %ecx
    call npx_report
    jmp 6b

/* cdecl: dst, src, count. Test 32-bit REP followed by absolute indexed JMP
 * through an image-loaded readonly table, including the zero-tail path.
 */
copy_dispatch:
    pushl %ebp
    movl %esp, %ebp
    pushl %esi
    pushl %edi
    movl 8(%ebp), %edi
    movl 12(%ebp), %esi
    movl 16(%ebp), %ecx
    movl %ecx, %edx
    andl $3, %edx
    shrl $2, %ecx
    rep movsl
    jmp *dispatch(,%edx,4)
tail3: movsb
tail2: movsb
tail1: movsb
tail0:
    popl %edi
    popl %esi
    popl %ebp
    ret
.balign 4
dispatch: .long tail0, tail1, tail2, tail3

.data
.balign 4
.globl thunks
thunks: .long 0
completed: .long 0
source: .space 80
destination: .space 80
