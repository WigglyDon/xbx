/* SPDX-License-Identifier: MIT
 * Original finite public-kernel observer ABI fixture. Synthetic data only.
 * Reuses the existing fixture's SEH CONTEXT layout; no kernel handler change.
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
    pushl $_Handler
    pushl %fs:0
    movl %esp, %fs:0
    movl %esp, _Stack
    /* The public I/O callback export must retain exact pointer identity. */
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
    cmpl _XkThunks+44, %eax
    jne .Lfail
    movl $0x11223344, %ebx
    movl $0x55667788, %esi
    movl $0x99aabbcc, %edi
    movl $0x13579bdf, %ebp
    call *_XkThunks+48       /* frequent omitted KeGetCurrentThread */
    testl %eax, %eax
    jz .Lfail
    call .Labi
    pushl $0x5842584f
    pushl $16
    call *_XkThunks+52       /* typed pointer result; never exposed raw */
    testl %eax, %eax
    jz .Lfail
    movl %eax, _Pool
    movl $0xa5a55a5a, (%eax)
    cmpl $0xa5a55a5a, (%eax)
    jne .Lfail
    call .Labi
    pushl _Pool
    call *_XkThunks+56       /* void result */
    call .Labi
    pushl $0x12345678
    pushl $64
    pushl $_Buffer
    call *_XkThunks+40       /* typed void fill */
    call .Labi
    pushl $0
    pushl $0
    pushl $_Ready
    call *_XkThunks
    call .Labi
    pushl $0
    pushl $0
    pushl $_Release
    call *_XkThunks
    call .Labi
    pushl $_Notify
    call *_XkThunks+4
    testl %eax, %eax
    jnz .Lfail
    call .Labi
    call .Lcreate
    pushl $_Timeout
    pushl $0
    pushl $0
    pushl $0
    pushl $_Ready
    call *_XkThunks+12
    testl %eax, %eax
    jnz .Lfail
    call .Labi
    /* Child atomically signalled Ready then entered the Release wait. Its
     * real in-flight return token will outlive its 1024-entry ring slot. */
    call .Lmany
    pushl $0
    pushl $0
    pushl $_Release
    call *_XkThunks+16
    call .Labi
    call .Ljoin
    cmpl $1, _ChildrenDone
    jne .Lfail
    /* A separate completed lifetime before capture: labels cannot recycle. */
    movl $1, _Phase
    call .Lcreate
    call .Ljoin
    cmpl $2, _ChildrenDone
    jne .Lfail
    /* Keep actual pointer/void type samples inside the final ring window. */
    pushl $0x5842584f
    pushl $16
    call *_XkThunks+52
    testl %eax, %eax
    jz .Lfail
    movl %eax, _Pool
    call .Labi
    pushl _Pool
    call *_XkThunks+56
    call .Labi
    pushl $0x12345678
    pushl $64
    pushl $_Buffer
    call *_XkThunks+40
    call .Labi
    movl $2, _Phase
    /* Notify runs synchronously in this parent while the outer create call is
     * in flight. Exactly one synthetic data read faults through normal SEH. */
    call .Lcreate
    cmpl $1, _Handled
    jne .Lfail
    call .Ljoin
    cmpl $3, _ChildrenDone
    jne .Lfail
    call .Lmany             /* >1024 later calls must not alter frozen rows */
    cmpl $0x10203040, _Before
    jne .Lfail
    cmpl $0x50607080, _After
    jne .Lfail
    movl $0x01020304, %ecx
    call *_XkThunks+36      /* success-only final marker, scalar unclassified */
    cmpl $0x04030201, %eax
    jne .Lfail
    call .Labi
.Ldone:
    hlt
    jmp .Ldone
.Lfail:
    pushl $0x12345678
    pushl $64
    pushl $_Buffer
    call *_XkThunks+20      /* different final ordinal identifies failure */
    jmp .Ldone
.Lcreate:
    /* Extra local return address is accounted for while checking stdcall40. */
    pushl $_Child
    pushl $0
    pushl $0
    pushl $0
    pushl $0
    pushl $0
    pushl $0
    pushl $0x4000
    pushl $0
    pushl $_Thread
    call *_XkThunks+8
    testl %eax, %eax
    jnz .Lfail
    call .Lnestedabi
    ret
.Ljoin:
    pushl $_Timeout
    pushl $0
    pushl _Thread
    call *_XkThunks+24
    testl %eax, %eax
    jnz .Lfail
    call .Lnestedabi
    pushl _Thread
    call *_XkThunks+28
    testl %eax, %eax
    jnz .Lfail
    call .Lnestedabi
    ret
.Lmany:
    movl $1100, _Iterations
.Lloop:
    pushl $0x12345678
    pushl $64
    pushl $_Buffer
    call *_XkThunks+20
    cmpl $64, %eax
    jne .Lfail
    call .Lnestedabi
    decl _Iterations
    jnz .Lloop
    ret
.Labi:
    leal 4(%esp), %eax
    jmp .Lcheck
.Lnestedabi:
    leal 8(%esp), %eax
.Lcheck:
    cmpl _Stack, %eax
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
_Notify:
    cmpl $0, 12(%esp)       /* Create flag */
    je .Lnotifyreturn
    cmpl $2, _Phase
    jne .Lnotifyreturn
    cmpl $0, _Handled
    jne .Lnotifyreturn
_Fault:
    movl 4, %eax           /* deterministic unmapped data read, not a CALL */
_AfterFault:
.Lnotifyreturn:
    ret $12
_Handler:
    movl 4(%esp), %eax
    cmpl $0xc0000005, (%eax)
    jne .Lsearch
    cmpl $_Fault, 12(%eax)
    jne .Lsearch
    cmpl $2, 16(%eax)
    jb .Lsearch
    cmpl $0, 20(%eax)      /* read */
    jne .Lsearch
    cmpl $4, 24(%eax)
    jne .Lsearch
    movl 12(%esp), %edx
    cmpl $_Fault, 0x224(%edx)
    jne .Lsearch
    movl $_AfterFault, 0x224(%edx)
    movl $1, _Handled
    xorl %eax, %eax
    ret
.Lsearch:
    movl $1, %eax
    ret
.safeseh _Handler
_Child:
    cmpl $0, _Phase
    jne .Lchilddone
    pushl $1              /* Wait=TRUE chains signal+wait atomically */
    pushl $0
    pushl $_Ready
    call *_XkThunks+16
    pushl $_Timeout
    pushl $0
    pushl $0
    pushl $0
    pushl $_Release
    call *_XkThunks+12
    testl %eax, %eax
    jnz .Lchildexit
.Lchilddone:
    incl _ChildrenDone
.Lchildexit:
    pushl $0
    call *_XkThunks+32
    jmp .Lfail
.data
.p2align 2
.globl _XkThunks
_XkThunks:
    .long 0x8000006c /*108 KeInitializeEvent */
    .long 0x80000101 /*257 PsSetCreateThreadNotifyRoutine */
    .long 0x800000ff /*255 PsCreateSystemThreadEx */
    .long 0x8000009f /*159 KeWaitForSingleObject */
    .long 0x80000091 /*145 KeSetEvent */
    .long 0x8000010d /*269 RtlCompareMemoryUlong */
    .long 0x800000e9 /*233 NtWaitForSingleObject */
    .long 0x800000bb /*187 NtClose */
    .long 0x80000102 /*258 PsTerminateSystemThread */
    .long 0x80000133 /*307 RtlUlongByteSwap */
    .long 0x8000011d /*285 RtlFillMemoryUlong */
    .long 0x800000e8 /*232 identity-sensitive callback */
    .long 0x80000068 /*104 omitted current-thread query */
    .long 0x8000000f /*15 ExAllocatePoolWithTag */
    .long 0x80000011 /*17 ExFreePool */
    .long 0
_Stack: .long 0
_Pool: .long 0
_Thread: .long 0
_Phase: .long 0
_Handled: .long 0
_ChildrenDone: .long 0
_Iterations: .long 0
_Timeout: .quad -50000000
_Ready: .space 16
_Release: .space 16
_Before: .long 0x10203040
_Buffer: .space 64
_After: .long 0x50607080
