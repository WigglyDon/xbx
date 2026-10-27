/* SPDX-License-Identifier: MIT
 * Original public file-read contract. Generated payload; no game code/data.
 * ABI: XboxDev/nxdk a6c31ee90540710a3df26a0712b61492f2ea03dd.
 * Page-aligned 16 KiB read, reported byte count, data integrity, then a
 * non-page-aligned seek/read across a sector. Separate writable data pages.
 */
.text
.globl _start
_start:
    cld
    call serial_init
    movl $boot,%esi
    call text
    movl $1,stage
    pushl $network_info
    pushl $attributes
    call *thunks+12
    call success
    cmpl $32772,network_info+40
    jne fail
    cmpl $0,network_info+44
    jne fail
    movl $2,stage
    pushl $0x64                 /* synchronous, non-directory, sequential */
    pushl $1                    /* FILE_OPEN */
    pushl $1                    /* FILE_SHARE_READ */
    pushl $0
    pushl $0
    pushl $iosb
    pushl $attributes
    pushl $0x80100080           /* GENERIC_READ | SYNCHRONIZE | READ_ATTRIBUTES */
    pushl $handle
    call *thunks+8
    call success
    movl $3,stage
    pushl $5                    /* FileStandardInformation */
    pushl $24
    pushl $standard_info
    pushl $iosb
    pushl handle
    call *thunks+16
    call success
    cmpl $32772,standard_info+8
    jne fail
    cmpl $0,standard_info+12
    jne fail
    movl $4,stage
    call seek
    movl $buffer,%edi
    movl $4096,%ecx
    movl $0xa5a5a5a5,%eax
    rep stosl
    movl $5,stage
    movl $16384,%ecx
    call read
    cmpl $16384,iosb+4
    jne fail
    movl $6,stage
    xorl %ebx,%ebx
    movl $4096,%ecx
    call verify
    movl $7,stage
    movl $2044,position
    call seek
    movl $12,%ecx
    call read
    cmpl $12,iosb+4
    jne fail
    movl $8,stage
    movl $511,%ebx
    movl $3,%ecx
    call verify
    movl $9,stage
    pushl handle
    call *thunks+4
    call success
    movl $passed,%esi
    call text
    jmp done
seek:
    pushl $14                   /* FilePositionInformation */
    pushl $8
    pushl $position
    pushl $iosb
    pushl handle
    call *thunks+20
    call success
    ret
read:
    movl $0xdead0001,iosb
    movl $0xdead0002,iosb+4
    pushl $0                    /* current file position */
    pushl %ecx
    pushl $buffer
    pushl $iosb
    pushl $0
    pushl $0
    pushl $0
    pushl handle
    call *thunks
    cmpl $0x103,%eax             /* complete a permitted pending read */
    jne 2f
    pushl $timeout
    pushl $0
    pushl handle
    call *thunks+24
    call success
    movl iosb,%eax
2:
    call success
    cmpl $0,iosb
    jne fail
    ret
success:
    movl %eax,status
    testl %eax,%eax
    jnz fail
    ret
verify:
    movl $buffer,%edi
1:  imull $0x01020305,%ebx,%eax
    xorl $0x6e000000,%eax
    cmpl %eax,(%edi)
    jne fail
    addl $4,%edi
    incl %ebx
    loop 1b
    ret
fail:
    movl $failed,%esi
    call text
    movl stage,%eax
    call hex
    movl $status_label,%esi
    call text
    movl status,%eax
    call hex
    movl $count_label,%esi
    call text
    movl iosb+4,%eax
    call hex
    movl $word_label,%esi
    call text
    movl buffer,%eax
    call hex
    movl $newline,%esi
    call text
done:
    sti
    hlt
    jmp done
boot: .asciz "XBX FILE READ BOOT\n"
passed: .asciz "XBX FILE READ PASS\n"
failed: .asciz "XBX FILE READ FAIL stage="
status_label: .asciz " status="
count_label: .asciz " count="
word_label: .asciz " first="
newline: .asciz "\n"
superio: .byte 7,4,0x60,3,0x61,0xf8,0x70,4,0x30,1
.data
.balign 4
.globl thunks
thunks:
    .long 0x800000db             /* 219 NtReadFile */
    .long 0x800000bb             /* 187 NtClose */
    .long 0x800000be             /* 190 NtCreateFile */
    .long 0x800000d2             /* 210 NtQueryFullAttributesFile */
    .long 0x800000d3             /* 211 NtQueryInformationFile */
    .long 0x800000e2             /* 226 NtSetInformationFile */
    .long 0x800000e9             /* 233 NtWaitForSingleObject */
    .long 0
handle: .long 0
stage: .long 0
status: .long 0
iosb: .long 0,0
position: .long 0,0
timeout: .quad -300000000
network_info: .space 56
standard_info: .space 24
attributes: .long 0,path_string,0x40
path_string: .short path_end-path-1,path_end-path
    .long path
path: .asciz "\\Device\\CdRom0\\payload.bin"
path_end:
.balign 4096
buffer: .space 16384
