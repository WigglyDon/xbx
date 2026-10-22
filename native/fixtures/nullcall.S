/* SPDX-License-Identifier: MIT
 * Mechanical i386 guest exception ABI fixture. Rust builds, runs and judges it.
 * No SDK, game code, firmware bytes, disk writes or fabricated QMP results.
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
    orl $1, _XkTitleResults+4
    /* EXCEPTION_REGISTRATION: Next, cdecl Handler. */
    pushl $_XbxExceptionHandler
    pushl %fs:0
    movl %esp, %fs:0
    xorl %eax, %eax
    call *%eax
.globl _XbxNullReturn
_XbxNullReturn:
    popl %eax
    movl %eax, %fs:0
    addl $4, %esp
    orl $4, _XkTitleResults+4
    movl $0x46584258, _XkTitleResults
1:  sti
    hlt
    jmp 1b

_XbxExceptionHandler:
    movl 4(%esp), %eax       /* EXCEPTION_RECORD */
    cmpl $0xc0000005, (%eax)
    jne 2f
    cmpl $0, 12(%eax)       /* ExceptionAddress */
    jne 2f
    movl 12(%esp), %edx     /* Xbox CONTEXT, layout from pinned kernel */
    cmpl $0, 0x224(%edx)    /* Eip */
    jne 2f
    movl 0x230(%edx), %eax  /* Esp */
    cmpl $_XbxNullReturn, (%eax)
    jne 2f
    orl $2, _XkTitleResults+4
    movl $_XbxNullReturn, 0x224(%edx)
    addl $4, 0x230(%edx)    /* consume the null CALL's pushed return PC */
    xorl %eax, %eax         /* ExceptionContinueExecution */
    ret
2:  movl $1, %eax           /* ExceptionContinueSearch; fail visibly */
    ret
.safeseh _XbxExceptionHandler

.data
.balign 4
.globl _XkThunks
_XkThunks: .long 0
.globl _XkTitleResults
_XkTitleResults: .long 0, 0
