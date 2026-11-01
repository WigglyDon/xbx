/* SPDX-License-Identifier: MIT
 * Original PCRTC event counter. Disable delivery, poll and acknowledge the
 * public VBlank status bit for one independent performance-counter second.
 * No game content or proprietary implementation is involved.
 */
.text
.globl _start
_start:
    cld
    call serial_init
    call *thunks+4
    testl %edx,%edx
    jnz fail
    cmpl $3375000,%eax
    jne fail
    pushl $4
    pushl $4096
    pushl $0xfd600000
    call *thunks+8
    testl %eax,%eax
    jz fail
    movl %eax,registers
command:
    movl $ready,%esi
    call text
1:  movw $0x3fd,%dx
    inb %dx,%al
    testb $1,%al
    jnz 2f
    sti
    hlt
    jmp 1b
2:  movw $0x3f8,%dx
    inb %dx,%al
    cmpb $'v',%al
    jne 1b
    movl registers,%edi
    movl 0x140(%edi),%eax
    movl %eax,enabled
    movl $0,0x140(%edi)
    movl $1,0x100(%edi)
    movl $0,count
    call *thunks
    movl %eax,start
3:  testl $1,0x100(%edi)
    jz 4f
    incl count
    movl $1,0x100(%edi)
4:  call *thunks
    subl start,%eax
    cmpl $3375000,%eax
    jb 3b
    movl enabled,%eax
    movl %eax,0x140(%edi)
    movl $result,%esi
    call text
    movl count,%eax
    call hex
    movl $newline,%esi
    call text
    jmp command
fail:
    movl $failed,%esi
    call text
5:  sti
    hlt
    jmp 5b
.data
.globl thunks
thunks:
    .long 0x8000007e /*126 KeQueryPerformanceCounter */
    .long 0x8000007f /*127 KeQueryPerformanceFrequency */
    .long 0x800000b1 /*177 MmMapIoSpace */
    .long 0
registers: .long 0
enabled: .long 0
count: .long 0
start: .long 0
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
ready: .asciz "XBX VBLANK READY\n"
result: .asciz "XBX VBLANK COUNT="
failed: .asciz "XBX VBLANK FAIL\n"
newline: .asciz "\n"
