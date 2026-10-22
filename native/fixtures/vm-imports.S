/* SPDX-License-Identifier: MIT
 * Original mechanical i386 title-import ABI fixture. Rust owns the build,
 * observation and acceptance policy. Interfaces: pinned public OXOS mm.h and
 * exports.tsv (allocate stdcall20, free stdcall12). Synthetic RAM only.
 */
.def @feat.00;
.scl 3;
.type 0;
.endef
.globl @feat.00
.set @feat.00, 1

.set R, _XkTitleResults
.set RESERVE, 0x10000
.set COMMIT, 0x2000
.set WORDS, COMMIT/4

.macro abi_begin before
    movl $0x11223344, %ebx
    movl $0x55667788, %esi
    movl $0x99aabbcc, %edi
    movl $0x13579bdf, %ebp
    movl %esp, R+\before
.endm

.macro abi_check before, after, regs, bit
    movl %esp, R+\after
    movl %ebx, R+\regs
    movl %esi, R+\regs+4
    movl %edi, R+\regs+8
    movl %ebp, R+\regs+12
    cmpl R+\before, %esp
    jne .Lstack_fail
    orl $\bit, R+12
    cmpl $0x11223344, %ebx
    jne .Lregister_fail
    cmpl $0x55667788, %esi
    jne .Lregister_fail
    cmpl $0x99aabbcc, %edi
    jne .Lregister_fail
    cmpl $0x13579bdf, %ebp
    jne .Lregister_fail
    orl $\bit, R+16
.endm

.text
.globl _XkTitleEntry
_XkTitleEntry:
    cld
    movl $1, R+4
    movl _XkThunks, %eax
    movl %eax, R+136
    movl _XkThunks+4, %eax
    movl %eax, R+140
    abi_begin 64
    pushl $4                    /* PAGE_READWRITE */
    pushl $0x2000               /* MEM_RESERVE */
    pushl $R+36                 /* RegionSize: 64 KiB */
    pushl $0                    /* ZeroBits */
    pushl $R+32                 /* BaseAddress initially NULL */
    call *_XkThunks
.globl _XbxVmReserveReturn
_XbxVmReserveReturn:
    movl %eax, R+20
    abi_check 64, 68, 88, 1
    testl %eax, %eax
    jne .Lreserve_status
    movl R+32, %eax
    cmpl $0x20000, %eax         /* beyond this tiny fixture image */
    jb .Lreserve_range
    cmpl $0x7ffd0000, %eax      /* room for the entire user reservation */
    ja .Lreserve_range
    testl $0xffff, %eax
    jne .Lreserve_range
    cmpl $RESERVE, R+36
    jne .Lreserve_range
    movl $2, R+4
    addl $0x1000, %eax          /* commit two interior pages */
    movl %eax, R+40
    abi_begin 72
    pushl $4
    pushl $0x1000               /* MEM_COMMIT, separate operation */
    pushl $R+44
    pushl $0
    pushl $R+40
    call *_XkThunks
.globl _XbxVmCommitReturn
_XbxVmCommitReturn:
    movl %eax, R+24
    abi_check 72, 76, 104, 2
    testl %eax, %eax
    jne .Lcommit_status
    movl R+32, %eax
    addl $0x1000, %eax
    cmpl %eax, R+40
    jne .Lcommit_range
    cmpl $COMMIT, R+44
    jne .Lcommit_range
    movl $3, R+4
    movl R+40, %esi
    xorl %ecx, %ecx
.Lzero:
    cmpl $0, (%esi,%ecx,4)
    jne .Lzero_fail
    incl %ecx
    movl %ecx, R+56
    cmpl $WORDS, %ecx
    jb .Lzero
    movl $4, R+4
    xorl %ecx, %ecx
.Lwrite:
    movl %ecx, %eax
    xorl $0x58425800, %eax
    movl %eax, (%esi,%ecx,4)
    incl %ecx
    cmpl $WORDS, %ecx
    jb .Lwrite
    xorl %ecx, %ecx
.Lread:
    movl %ecx, %eax
    xorl $0x58425800, %eax
    cmpl %eax, (%esi,%ecx,4)
    jne .Lread_fail
    incl %ecx
    movl %ecx, R+60
    cmpl $WORDS, %ecx
    jb .Lread
    movl $5, R+4
    movl R+32, %eax
    movl %eax, R+48
    abi_begin 80
    pushl $0x8000               /* MEM_RELEASE, size zero: entire reservation */
    pushl $R+52
    pushl $R+48
    call *_XkThunks+4
    movl %eax, R+28
    abi_check 80, 84, 120, 4
    testl %eax, %eax
    jne .Lfree_status
    movl R+32, %eax
    cmpl %eax, R+48
    jne .Lfree_range
    cmpl $RESERVE, R+52
    jne .Lfree_range
    movl $6, R+4
    jmp .Lfinish

.Lreserve_status: movl $1, R+8; jmp .Lfinish
.Lreserve_range:  movl $2, R+8; jmp .Lfinish
.Lstack_fail:     movl $3, R+8; jmp .Lfinish
.Lregister_fail:  movl $4, R+8; jmp .Lfinish
.Lcommit_status:  movl $5, R+8; jmp .Lfinish
.Lcommit_range:   movl $6, R+8; jmp .Lfinish
.Lzero_fail:      movl $7, R+8; jmp .Lfinish
.Lread_fail:      movl $8, R+8; jmp .Lfinish
.Lfree_status:    movl $9, R+8; jmp .Lfinish
.Lfree_range:     movl $10, R+8
.Lfinish:
    movl $0x4d564258, R          /* completion, including explicit failure */
.Lidle:
    sti
    hlt
    jmp .Lidle

.data
.balign 4
.globl _XkThunks
_XkThunks: .long 0x800000b8, 0x800000c7, 0 /* ordinals 184 and 199 */
.globl _XkTitleResults
_XkTitleResults:
    /* magic, stage, failure, stack mask, register mask, three statuses */
    .long 0, 0, 0, 0, 0, -1, -1, -1
    /* reserve base/size, commit base/size, release base/size, loop counts */
    .long 0, RESERVE, 0, COMMIT, 0, 0, 0, 0
    /* three before/after ESP pairs, twelve actual preserved-register values */
    .fill 18, 4, 0
    /* observed loader-resolved imports */
    .long 0, 0
