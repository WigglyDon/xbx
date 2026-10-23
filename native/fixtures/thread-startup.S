/* SPDX-License-Identifier: MIT
 * Original finite loader-resolved thread/TLS contract, synthetic data only.
 * Public ABI facts: nxdk a6c31ee / OXOS d87cfc33; no title-derived code.
 * Two sequential children, one suspended: supplied system/start/context args,
 * zeroed TLS, caller-owned self-reference/template, alignment, stack bounds,
 * current-thread identity, resume/wait/close and parent stdcall40 balance.
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
    movl %esp, _SavedStack
    movl $0x11223344, %ebx
    movl $0x55667788, %esi
    movl $0x99aabbcc, %edi
    movl $0x13579bdf, %ebp
.Lcreate:
    movl $1, _Failure
    pushl $_SystemRoutine
    pushl $0
    pushl _Suspended
    pushl $_Context
    pushl $_StartRoutine
    pushl $_ThreadId
    pushl $36                 /* rounded32 + self-reference4 */
    pushl $0x4000
    pushl $16                 /* bounded object extension */
    pushl $_ThreadHandle
    call *_XkThunks
    testl %eax, %eax
    jnz .Lfail
    call .Labi
    cmpl $0, _ThreadHandle
    je .Lfail
    cmpl $0, _ThreadId
    je .Lfail
    cmpl $0, _Suspended
    je .Lwait
    movl $2, _Failure
    cmpl $0, _ChildStage
    jne .Lfail
    pushl $_PreviousSuspendCount
    pushl _ThreadHandle
    call *_XkThunks+4
    testl %eax, %eax
    jnz .Lfail
    call .Labi
    cmpl $1, _PreviousSuspendCount
    jne .Lfail
.Lwait:
    movl $3, _Failure
    pushl $_Timeout
    pushl $0
    pushl _ThreadHandle
    call *_XkThunks+8
    testl %eax, %eax
    jnz .Lfail
    call .Labi
    cmpl $0x600dcafe, _ChildStage
    jne .Lfail
    movl $4, _Failure
    pushl _ThreadHandle
    call *_XkThunks+12
    testl %eax, %eax
    jnz .Lfail
    call .Labi
    cmpl $0, _Suspended
    je .Lpass
    movl $0, _Suspended
    movl $0, _ChildStage
    movl $0x12345678, _Context
    jmp .Lcreate
.Lpass:
    movl $0x544c5301, %ecx
    call *_XkThunks+24
    jmp .Ldone
.Labi:
    leal 4(%esp), %eax
    cmpl _SavedStack, %eax
    jne .Labifail
    cmpl $0x11223344, %ebx
    jne .Labifail
    cmpl $0x55667788, %esi
    jne .Labifail
    cmpl $0x99aabbcc, %edi
    jne .Labifail
    cmpl $0x13579bdf, %ebp
    jne .Labifail
    ret
.Labifail:
    movl $5, _Failure
.Lfail:
    movl _Failure, %ecx
    orl $0xbad00000, %ecx
    call *_XkThunks+24
.Ldone:
    hlt
    jmp .Ldone

_SystemRoutine:
    cld
    movl $0xbad00101, _ChildStage
    cmpl $_StartRoutine, 4(%esp)
    jne .Lchildexit
    cmpl $_Context, 8(%esp)
    jne .Lchildexit
    call *_XkThunks+16        /* KeGetCurrentThread */
    movl $0xbad00102, _ChildStage
    testl %eax, %eax
    jz .Lchildexit
    cmpl %fs:0x28, %eax
    jne .Lchildexit
    movl 0x28(%eax), %esi    /* public KTHREAD.TlsData */
    testl %esi, %esi
    jz .Lchildexit
    movl $0xbad00103, _ChildStage
    cmpl 0x20(%eax), %esi    /* StackLimit < TLS */
    jbe .Lchildexit
    leal 36(%esi), %edx
    cmpl 0x1c(%eax), %edx    /* TLS end < StackBase */
    jae .Lchildexit
    cmpl %fs:4, %edx        /* PCR stack top excludes NPX, includes TLS */
    jne .Lchildexit
    movl %fs:8, %edx
    cmpl 0x20(%eax), %edx
    jne .Lchildexit
    leal 4(%esi), %edx
    testl $15, %edx
    jnz .Lchildexit
    movl $9, %ecx
    movl %esi, %edi
    xorl %eax, %eax
    repe scasl
    jne .Lchildexit
    /* This storage initialization belongs to this guest's startup routine. */
    movl %edx, (%esi)
    movl $8, %ecx
    movl %edx, %edi
    movl _Context, %eax
    rep stosl
    movl %esp, _ChildStack
    pushl 8(%esp)
    call *8(%esp)            /* supplied StartRoutine(StartContext) */
    movl $0xbad00104, _ChildStage
    cmpl _ChildStack, %esp
    jne .Lchildexit
    cmpl $0x600dcafe, %eax
    jne .Lchildexit
    movl %esi, %edx
    subl %fs:4, %edx
    /* Negative DWORD TLS index used by public caller-owned TLS startup. */
    sarl $2, %edx
    movl %fs:4, %edi
    movl (%edi,%edx,4), %edi
    leal 4(%esi), %edx
    cmpl %edx, %edi
    jne .Lchildexit
    movl $8, %ecx
    movl _Context, %eax
    repe scasl
    jne .Lchildexit
    movl $0x600dcafe, _ChildStage
.Lchildexit:
    pushl _ChildStage
    call *_XkThunks+20       /* PsTerminateSystemThread; must never return */
    movl $0xbad00105, _ChildStage
    jmp .Ldone
_StartRoutine:
    movl 4(%esp), %eax
    cmpl $_Context, %eax
    jne .Lstartfail
    movl $0x600dcafe, %eax
    ret $4
.Lstartfail:
    xorl %eax, %eax
    ret $4
.data
.p2align 2
.globl _XkThunks
_XkThunks:
    .long 0x800000ff /*255 PsCreateSystemThreadEx stdcall40 */
    .long 0x800000e0 /*224 NtResumeThread stdcall8 */
    .long 0x800000e9 /*233 NtWaitForSingleObject stdcall12 */
    .long 0x800000bb /*187 NtClose stdcall4 */
    .long 0x80000068 /*104 KeGetCurrentThread stdcall0 */
    .long 0x80000102 /*258 PsTerminateSystemThread stdcall4 */
    .long 0x80000133 /*307 RtlUlongByteSwap fastcall4: final sentinel */
    .long 0
_SavedStack: .long 0
_ChildStack: .long 0
_ThreadHandle: .long 0
_ThreadId: .long 0
_PreviousSuspendCount: .long 0
_Suspended: .long 1
_ChildStage: .long 0
_Failure: .long 0
_Context: .long 0x89abcdef
_Timeout: .quad -20000000  /* bounded two-second relative wait */
