/* SPDX-License-Identifier: MIT
 * Original mechanical loaded-title check. Rust owns image layout/build/acceptance.
 * Only synthetic data, zero-fill, and shared-page reference counters are read.
 */
.set R, 0x12000
.text
.globl _XkTitleEntry
_XkTitleEntry:
    cld
    movl $1, R+4
    xorl %ecx, %ecx
.La:
    cmpl $0x11223344, 0x11800(,%ecx,4)
    jne .Lfail_a
    incl %ecx
    movl %ecx, R+12
    cmpl $512, %ecx
    jb .La
    xorl %ecx, %ecx
.Lza:
    cmpl $0, 0x12020(,%ecx,4)
    jne .Lfail_za
    incl %ecx
    movl %ecx, R+16
    cmpl $504, %ecx
    jb .Lza
    xorl %ecx, %ecx
.Lb:
    cmpl $0x55667788, 0x12800(,%ecx,4)
    jne .Lfail_b
    incl %ecx
    movl %ecx, R+20
    cmpl $64, %ecx
    jb .Lb
    xorl %ecx, %ecx
.Lzb:
    cmpl $0, 0x12900(,%ecx,4)
    jne .Lfail_zb
    incl %ecx
    movl %ecx, R+24
    cmpl $448, %ecx
    jb .Lzb
    cmpw $2, 0x10402
    jne .Lfail_refs
    cmpw $2, 0x10404
    jne .Lfail_refs
    movl $1, R+28
    jmp .Ldone
.Lfail_a: movl $1, R+8; jmp .Ldone
.Lfail_za: movl $2, R+8; jmp .Ldone
.Lfail_b: movl $3, R+8; jmp .Ldone
.Lfail_zb: movl $4, R+8; jmp .Ldone
.Lfail_refs: movl $5, R+8
.Ldone:
    movl $0x4c584258, R
.Lidle:
    sti
    hlt
    jmp .Lidle
