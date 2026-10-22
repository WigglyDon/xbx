/* SPDX-License-Identifier: MIT
 * Original public-kernel-call journal regression, synthetic data only.
 * Three loader-resolved calls; verifies stdcall/fastcall, stack, preserved
 * registers, bounded fill/read, and canaries before publishing final sentinel.
 */
.def @feat.00;
.scl 3;
.type 0;
.endef
.globl @feat.00
.set @feat.00, 1
.text
.globl _XkTitleEntry
_XkTitleEntry:
    cld
    /* NtUserIoApcDispatcher is also an API callback. The I/O manager uses
     * its exact identity to select user-mode delivery; it must not be wrapped.
     * Read only our public kernel's PE export directory, never another title.
     */
    movl $0x80010000, %eax
    movl 0x3c(%eax), %edx
    addl %eax, %edx
    movl 0x78(%edx), %edx
    addl %eax, %edx
    movl $232, %ecx
    subl 0x10(%edx), %ecx
    movl 0x1c(%edx), %edx
    addl %eax, %edx
    addl (%edx,%ecx,4), %eax
    cmpl _XkThunks+12, %eax
    jne .Lfail
    movl %esp, _SavedStack
    movl $0x11223344, %ebx
    movl $0x55667788, %esi
    movl $0x99aabbcc, %edi
    movl $0x13579bdf, %ebp
    pushl $0x12345678
    pushl $64
    pushl $_Buffer
    call *_XkThunks
    call .Labi
    pushl $0x12345678
    pushl $64
    pushl $_Buffer
    call *_XkThunks+4
    cmpl $64, %eax
    jne .Lfail
    call .Labi
    cmpl $0x10203040, _CanaryBefore
    jne .Lfail
    cmpl $0x50607080, _CanaryAfter
    jne .Lfail
    movl $0x01020304, %ecx
    call *_XkThunks+8
    cmpl $0x04030201, %eax
    jne .Lfail
    call .Labi
.Ldone:
    hlt
    jmp .Ldone
.Labi:
    leal 4(%esp), %eax
    cmpl _SavedStack, %eax
    jne .Lfail
    cmpl $0x11223344, %ebx
    jne .Lfail
    cmpl $0x55667788, %esi
    jne .Lfail
    cmpl $0x99aabbcc, %edi
    jne .Lfail
    cmpl $0x13579bdf, %ebp
    jne .Lfail
    ret
.Lfail:
    movl $0xbad00000, %ecx
    call *_XkThunks+8
    jmp .Ldone
.data
.p2align 2
.globl _XkThunks
_XkThunks:
    .long 0x8000011d /* 285 RtlFillMemoryUlong, stdcall12 */
    .long 0x8000010d /* 269 RtlCompareMemoryUlong, stdcall12 */
    .long 0x80000133 /* 307 RtlUlongByteSwap, fastcall4 */
    .long 0x800000e8 /* 232 NtUserIoApcDispatcher: pointer identity */
    .long 0
_SavedStack: .long 0
_CanaryBefore: .long 0x10203040
_Buffer: .space 64
_CanaryAfter: .long 0x50607080
