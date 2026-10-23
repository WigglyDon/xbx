/* SPDX-License-Identifier: MIT
 * Original synthetic lazy-FPU ownership regression. Public kernel ABIs only.
 * A saves its x87/SSE state, B owns live x87/SSE while blocked, A restores
 * without first using FP, then B must recover its own values after switching.
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
    pushl $0
    pushl $0                 /* notification event */
    pushl $_ChildReady
    call *_XkThunks
    pushl $0
    pushl $0
    pushl $_ParentRestored
    call *_XkThunks
    fldl _ParentValue
    movaps _ParentPattern, %xmm0
    movl $1, _Failure
    pushl $_FloatSave
    call *_XkThunks+4
    testl %eax, %eax
    jnz .Lfail
    pushl $_ChildRoutine
    pushl $0
    pushl $0
    pushl $0
    pushl $0
    pushl $0
    pushl $0
    pushl $0x4000
    pushl $0
    pushl $_ThreadHandle
    call *_XkThunks+8
    movl $2, _Failure
    testl %eax, %eax
    jnz .Lfail
    pushl $_Timeout
    pushl $0
    pushl $0
    pushl $0
    pushl $_ChildReady
    call *_XkThunks+12
    movl $3, _Failure
    testl %eax, %eax
    jnz .Lfail
    /* No FP instruction may precede this restore after B becomes the owner. */
    pushl $_FloatSave
    call *_XkThunks+16
    movl $4, _Failure
    testl %eax, %eax
    jnz .Lfail
    fstpl _ParentObserved
    movaps %xmm0, _ObservedPattern
    cmpl $0, _ParentObserved
    jne .Lfail
    cmpl $0x3ff40000, _ParentObserved+4
    jne .Lfail
    movl $_ObservedPattern, %esi
    movl $_ParentPattern, %edi
    movl $4, %ecx
    repe cmpsl
    jne .Lfail
    pushl $0
    pushl $0
    pushl $_ParentRestored
    call *_XkThunks+20
    pushl $_Timeout
    pushl $0
    pushl _ThreadHandle
    call *_XkThunks+24
    movl $5, _Failure
    testl %eax, %eax
    jnz .Lfail
    cmpl $0x600dcafe, _ChildResult
    jne .Lfail
    pushl _ThreadHandle
    call *_XkThunks+28
    movl $6, _Failure
    testl %eax, %eax
    jnz .Lfail
    movl $0x4e505801, %ecx
    call *_XkThunks+36
    jmp .Ldone
.Lfail:
    movl _Failure, %ecx
    orl $0xbad00200, %ecx
    call *_XkThunks+36
.Ldone:
    hlt
    jmp .Ldone

_ChildRoutine:
    cld
    fldl _ChildValue
    movaps _ChildPattern, %xmm0
    pushl $0
    pushl $0
    pushl $_ChildReady
    call *_XkThunks+20
    pushl $_Timeout
    pushl $0
    pushl $0
    pushl $0
    pushl $_ParentRestored
    call *_XkThunks+12
    movl $0xbad00301, _ChildResult
    testl %eax, %eax
    jnz .Lexit
    fstpl _ChildObserved
    movaps %xmm0, _ObservedPattern
    movl $0xbad00302, _ChildResult
    cmpl $0, _ChildObserved
    jne .Lexit
    cmpl $0x40040000, _ChildObserved+4
    jne .Lexit
    movl $_ObservedPattern, %esi
    movl $_ChildPattern, %edi
    movl $4, %ecx
    repe cmpsl
    jne .Lexit
    movl $0x600dcafe, _ChildResult
.Lexit:
    /* Synthetic result exposed as a call result through the bounded journal. */
    movl _ChildResult, %ecx
    call *_XkThunks+36
    pushl _ChildResult
    call *_XkThunks+32
    jmp .Ldone
.data
.p2align 2
.globl _XkThunks
_XkThunks:
    .long 0x8000006c /*108 KeInitializeEvent stdcall12 */
    .long 0x8000008e /*142 KeSaveFloatingPointState stdcall4 */
    .long 0x800000ff /*255 PsCreateSystemThreadEx stdcall40 */
    .long 0x8000009f /*159 KeWaitForSingleObject stdcall20 */
    .long 0x8000008b /*139 KeRestoreFloatingPointState stdcall4 */
    .long 0x80000091 /*145 KeSetEvent stdcall12 */
    .long 0x800000e9 /*233 NtWaitForSingleObject stdcall12 */
    .long 0x800000bb /*187 NtClose stdcall4 */
    .long 0x80000102 /*258 PsTerminateSystemThread stdcall4 */
    .long 0x80000133 /*307 RtlUlongByteSwap fastcall4 */
    .long 0
_ThreadHandle: .long 0
_Failure: .long 0
_ChildResult: .long 0
_Timeout: .quad -20000000
_ParentValue: .quad 0x3ff4000000000000 /*1.25 */
_ChildValue: .quad 0x4004000000000000 /*2.5 */
_ParentObserved: .quad 0
_ChildObserved: .quad 0
_ChildReady: .space 16
_ParentRestored: .space 16
_FloatSave: .space 24
.p2align 4
_ParentPattern: .long 0x12345678,0x23456789,0x3456789a,0x456789ab
_ChildPattern: .long 0x87654321,0x76543210,0x6543210f,0x543210fe
_ObservedPattern: .space 16
