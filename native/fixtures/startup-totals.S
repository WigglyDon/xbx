/* SPDX-License-Identifier: MIT
 * Original early-result controls, followed by the retained original observer
 * fixture. No game-derived code or new kernel imports. The huge allocation is
 * intentionally beyond 64 MiB; the invalid handle call returns without waiting.
 */
#define _XkTitleEntry _XkObserverEntry
#include "observer.S"
#undef _XkTitleEntry

.text
.globl _XkTitleEntry
_XkTitleEntry:
    cld
    movl %esp, _TotalsStack
    pushl $0x58425431
    pushl $0x40000000
    call *_XkThunks+52     /* ExAllocatePoolWithTag: null, no backing allocation */
    testl %eax, %eax
    jnz .Ltotalsfail
    cmpl _TotalsStack, %esp
    jne .Ltotalsfail
    pushl $_Timeout
    pushl $0
    pushl $0
    call *_XkThunks+24     /* NtWaitForSingleObject: invalid handle */
    cmpl $0xc0000008, %eax
    jne .Ltotalsfail
    cmpl _TotalsStack, %esp
    jne .Ltotalsfail
    jmp _XkObserverEntry
.Ltotalsfail:
    ud2                   /* never emit the original success-only final marker */
    jmp .Ltotalsfail
.data
.p2align 2
_TotalsStack: .long 0
