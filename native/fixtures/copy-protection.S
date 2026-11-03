/* SPDX-License-Identifier: MIT
 * Original effective-page-protection gate for copy-dispatch.S.
 * Public Xbox NtQueryVirtualMemory: stdcall(BaseAddress, Info), ordinal 217.
 * The two envelopes differ only in their section head/tail readonly flags.
 */
.text
check_copy_protection:
    pushl $copy_memory_info
    pushl $dispatch
    call *thunks
    movl %eax, %ecx
    call npx_report
    testl %eax, %eax
    jnz failed
    movl copy_memory_info+20, %ecx
    call npx_report
    cmpl $0x11000, copy_memory_info
    jne failed
    cmpl $0x1000, copy_memory_info+16
    jne failed
    cmpl $0x1000, copy_memory_info+12
    jb failed
    movl $4, %eax
    testl $0x10, 0x10348
    jz 1f
    movl $2, %eax
1:  cmpl %eax, copy_memory_info+20
    jne failed
    ret
.data
.balign 4
copy_memory_info: .space 28
