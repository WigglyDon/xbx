/* SPDX-License-Identifier: MIT
 * Original public file-read contract. Generated payload; no game code/data.
 * ABI: XboxDev/nxdk a6c31ee90540710a3df26a0712b61492f2ea03dd.
 * Optional cache control creates only its original file in fresh storage.
 * Page-aligned 16 KiB read, reported byte count, data integrity, then a
 * non-page-aligned seek/read across a sector. Separate writable data pages.
 */
.ifndef XBX_HIGH_VA
.set XBX_HIGH_VA, 0
.endif
.ifndef XBX_DEMAND_BUFFER
.set XBX_DEMAND_BUFFER, 0
.endif
.ifndef XBX_IO_ERRORS_CONTROL
.set XBX_IO_ERRORS_CONTROL, 0
.endif
.if XBX_HIGH_VA
.if XBX_DEMAND_BUFFER
.set buffer, 0x05001000
.else
.set buffer, 0x05000000
.endif
.endif
.if !XBX_DEMAND_BUFFER
.set write_buffer, buffer
.endif
.if XBX_CACHE_TEST
.set EXPECTED_BYTES, 0x1200004
.else
.set EXPECTED_BYTES, 32772
.endif
.text
.globl _start
_start:
    cld
    call serial_init
    movl $boot,%esi
    call text
    .if XBX_HIGH_VA
    movl $16,stage
    pushl $4                    /* PAGE_READWRITE */
    .if XBX_DEMAND_BUFFER
    pushl $0x2000               /* MEM_RESERVE: the original handler commits on access. */
    .else
    pushl $0x3000               /* MEM_RESERVE | MEM_COMMIT */
    .endif
    pushl $heap_bytes
    pushl $0
    pushl $heap_base
    call *thunks+36             /* NtAllocateVirtualMemory */
    call success
    .if XBX_DEMAND_BUFFER
    cmpl $buffer-4096,heap_base
    jne fail
    cmpl $20480,heap_bytes
    .else
    cmpl $buffer,heap_base
    jne fail
    cmpl $16384,heap_bytes
    .endif
    jne fail
    .endif
    .if XBX_DEMAND_BUFFER
    pushl $demand_handler
    pushl %fs:0
    movl %esp,%fs:0
    movl %esp,seh_frame
    movl $18,stage
    movb $0xa5,buffer-4096       /* Validate the handler and create the shared page table. */
    cmpl $1,demand_faults
    jne fail
    pushl $buffer-4096
    call *thunks+40
    testb %al,%al
    jz fail
    movl $0,demand_faults
    movl $demand_header_passed,%esi
    call text
    .endif
    .if XBX_CACHE_TEST
    call prepare_cache
    .endif
    movl $1,stage
    pushl $network_info
    pushl $attributes
    call *thunks+12
    call success
    cmpl $EXPECTED_BYTES,network_info+40
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
    cmpl $EXPECTED_BYTES,standard_info+8
    jne fail
    cmpl $0,standard_info+12
    jne fail
    movl $4,stage
    call seek
    .if !XBX_DEMAND_BUFFER
    movl $buffer,%edi
    movl $4096,%ecx
    movl $0xa5a5a5a5,%eax
    rep stosl
    .endif
    movl $5,stage
    movl $16384,%ecx
    call read
    cmpl $16384,iosb+4
    jne fail
    .if XBX_DEMAND_BUFFER
    movl $17,stage
    cmpl $4,demand_faults       /* All four pages must become resident during I/O. */
    jne fail
    movl $buffer,%ebx
1:  pushl %ebx
    call *thunks+40             /* MmIsAddressValid, without touching the buffer. */
    testb %al,%al
    jz fail
    addl $4096,%ebx
    cmpl $buffer+16384,%ebx
    jb 1b
    movl $demand_passed,%esi
    call text
    .endif
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
    .if XBX_IO_ERRORS_CONTROL
    movl $19,stage
    pushl $0x64
    pushl $1                    /* FILE_OPEN of an absent original name. */
    pushl $1
    pushl $0
    pushl $0
    pushl $iosb
    pushl $missing_attributes
    pushl $0x80100080
    pushl $missing_handle
    call *thunks+8
    movl %eax,status
    cmpl $0xc0000034,%eax       /* STATUS_OBJECT_NAME_NOT_FOUND */
    jne fail
    pushl %eax
    movl $missing_label,%esi
    call text
    popl %eax
    call hex
    movl $newline,%esi
    call text
    movl $20,stage
    movl $EXPECTED_BYTES,position
    movl $0,position+4
    pushl $position             /* Explicit offset at EOF. */
    pushl $2048
    pushl $buffer
    pushl $iosb
    pushl $0
    pushl $0
    pushl $0
    pushl handle
    call *thunks
    movl %eax,status
    cmpl $0xc0000011,%eax       /* STATUS_END_OF_FILE */
    jne fail
    pushl %eax
    movl $eof_label,%esi
    call text
    popl %eax
    call hex
    movl $newline,%esi
    call text
    movl $expected_errors_passed,%esi
    call text
    .endif
    movl $9,stage
    pushl handle
    call *thunks+4
    call success
    movl $data_passed,%esi
    call text
    .if XBX_CACHE_TEST
    movl flush_status,%eax
    testl %eax,%eax
    jz 3f
    movl $15,stage
    movl %eax,status
    jmp fail
3:
    .endif
    movl $passed,%esi
    call text
    jmp done
/* Original data only. FILE_CREATE refuses an existing path. */
.if XBX_CACHE_TEST
prepare_cache:
    movl $10,stage
    pushl $0x64
    pushl $2                    /* FILE_CREATE */
    pushl $0
    pushl $0
    pushl $0
    pushl $iosb
    pushl $attributes
    pushl $0xc0100080           /* read/write, synchronize, attributes */
    pushl $handle
    call *thunks+8
    call success
    movl $11,stage
    pushl $20                   /* FileEndOfFileInformation */
    pushl $8
    pushl $cache_extent
    pushl $iosb
    pushl handle
    call *thunks+20
    call success
    xorl %ebx,%ebx
    movl $12,stage
    call fill
    movl $16384,%ecx
    call write
    movl $13,stage
    call fill
    movl $16384,%ecx
    call write
    movl $14,stage
    call fill
    movl $4,%ecx
    call write
    movl $15,stage
    pushl $iosb
    pushl handle
    call *thunks+32             /* NtFlushBuffersFile */
    movl %eax,flush_status
    /* Retain flush failure, but independently finish the requested read proof. */
    pushl %eax
    movl $flush_label,%esi
    call text
    popl %eax
    call hex
    movl $newline,%esi
    call text
    pushl handle
    call *thunks+4
    call success
    ret
fill:
    movl $write_buffer,%edi
    movl $4096,%ecx
1:  imull $0x01020305,%ebx,%eax
    xorl $0x6e000000,%eax
    stosl
    incl %ebx
    loop 1b
    ret
write:
    movl %ecx,requested
    pushl $0
    pushl %ecx
    pushl $write_buffer
    pushl $iosb
    pushl $0
    pushl $0
    pushl $0
    pushl handle
    call *thunks+28             /* NtWriteFile */
    cmpl $0x103,%eax
    jne 2f
    pushl $timeout
    pushl $0
    pushl handle
    call *thunks+24
    call success
    movl iosb,%eax
2:  call success
    cmpl $0,iosb
    jne fail
    movl requested,%eax
    cmpl %eax,iosb+4
    jne fail
    ret
.endif
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
    .if XBX_DEMAND_BUFFER
    xorl %eax,%eax             /* A failed observation must not fault in its own data. */
    .else
    .if XBX_HIGH_VA
    xorl %eax,%eax
    cmpl $16,stage
    je 8f                       /* Allocation failure must not read an unmapped buffer. */
    .endif
    movl buffer,%eax
    .endif
8:  call hex
    .if XBX_DEMAND_BUFFER
    movl $fault_label,%esi
    call text
    movl demand_faults,%eax
    call hex
    .endif
    movl $newline,%esi
    call text
done:
    .if XBX_DEMAND_BUFFER
    movl seh_frame,%eax
    testl %eax,%eax
    jz 7f
    movl (%eax),%edx
    movl %edx,%fs:0
    movl $0,seh_frame
7:
    .endif
    sti
    hlt
    jmp done
boot: .asciz "XBX FILE READ BOOT\n"
passed: .asciz "XBX FILE READ PASS\n"
data_passed: .asciz "XBX FILE READ DATA PASS\n"
flush_label: .asciz "XBX FILE READ FLUSH STATUS="
failed: .asciz "XBX FILE READ FAIL stage="
status_label: .asciz " status="
count_label: .asciz " count="
word_label: .asciz " first="
newline: .asciz "\n"
.if XBX_IO_ERRORS_CONTROL
missing_label: .asciz "XBX FILE READ MISSING STATUS="
eof_label: .asciz "XBX FILE READ EOF STATUS="
expected_errors_passed: .asciz "XBX FILE READ EXPECTED ERRORS PASS\n"
.endif
.if XBX_DEMAND_BUFFER
demand_passed: .asciz "XBX FILE READ DEMAND PAGES PASS\n"
demand_header_passed: .asciz "XBX FILE READ DEMAND HEADER PASS\n"
fault_label: .asciz " faults="
.endif
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
    .long 0x800000ec             /* 236 NtWriteFile */
    .long 0x800000c6             /* 198 NtFlushBuffersFile */
    .if XBX_HIGH_VA
    .long 0x800000b8             /* 184 NtAllocateVirtualMemory */
    .endif
    .if XBX_DEMAND_BUFFER
    .long 0x800000ae             /* 174 MmIsAddressValid */
    .long 0x80000067             /* 103 KeGetCurrentIrql */
    .endif
    .long 0
heap_base: .long 0x05000000
heap_bytes:
.if XBX_DEMAND_BUFFER
    .long 20480
.else
    .long 16384
.endif
.if XBX_DEMAND_BUFFER
seh_frame: .long 0
demand_faults: .long 0
.endif
handle: .long 0
stage: .long 0
status: .long 0
flush_status: .long 0
iosb: .long 0,0
position: .long 0,0
cache_extent: .quad EXPECTED_BYTES
requested: .long 0
timeout: .quad -300000000
network_info: .space 56
standard_info: .space 24
attributes: .long 0,path_string,0x40
path_string: .short path_end-path-1,path_end-path
    .long path
path:
.if XBX_CACHE_TEST
    .asciz "\\Device\\Harddisk0\\Partition5\\xbx-original-read.bin"
.else
    .asciz "\\Device\\CdRom0\\payload.bin"
.endif
path_end:
.if XBX_IO_ERRORS_CONTROL
.balign 4
missing_handle: .long 0
missing_attributes: .long 0,missing_string,0x40
missing_string: .short missing_end-missing_path-1,missing_end-missing_path
    .long missing_path
missing_path: .asciz "\\Device\\CdRom0\\xbx-absent-original.bin"
missing_end:
.endif
.balign 4096
.if !XBX_HIGH_VA
buffer: .space 16384
.endif
.if XBX_DEMAND_BUFFER
write_buffer: .space 16384
.endif
