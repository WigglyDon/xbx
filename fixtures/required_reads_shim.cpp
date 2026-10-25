// SPDX-License-Identifier: MIT
// Mechanical host adapter for the exact extracted public loader functions.
// Only host pointers are widened; counts/statuses remain 32-bit. Not an i386 ABI test.
#include <cstdint>
#include <cstring>
using ULONG = uint32_t;
using USHORT = uint16_t;
using UCHAR = uint8_t;
using NTSTATUS = int32_t;
using SIZE_T = uint32_t;
using HANDLE = void *;
using PVOID = void *;
using PUCHAR = uint8_t *;
struct LARGE_INTEGER { int64_t QuadPart; };
struct IO_STATUS_BLOCK { NTSTATUS Status; ULONG Information; };
struct XBE_SECTION_HEADER {
    ULONG Flags;
    uintptr_t VirtualAddress;
    ULONG VirtualSize, PointerToRawData, SizeOfRawData, SectionName, SectionReferenceCount;
    USHORT *HeadSharedPageReferenceCount, *TailSharedPageReferenceCount;
    UCHAR SectionDigest[20];
};
constexpr NTSTATUS STATUS_SUCCESS = 0;
[[maybe_unused]] constexpr NTSTATUS STATUS_END_OF_FILE = static_cast<NTSTATUS>(0xc0000011u);
#define NT_SUCCESS(s) ((s) >= 0)
extern "C" NTSTATUS fixture_read(PVOID, ULONG, ULONG *);
static NTSTATUS NtReadFile(HANDLE, PVOID, PVOID, PVOID, IO_STATUS_BLOCK *io, PVOID dst, ULONG len, LARGE_INTEGER *) {
    io->Status = fixture_read(dst, len, &io->Information);
    return io->Status;
}
static NTSTATUS MiCommitImageMemory(PVOID *, SIZE_T *) { return 0; }
static void RtlZeroMemory(PVOID p, SIZE_T n) { std::memset(p, 0, n); }
static void XcSHAInit(PUCHAR) {}
static void XcSHAUpdate(PUCHAR, const UCHAR *, ULONG) {}
static void XcSHAFinal(PUCHAR, PUCHAR digest) { std::memset(digest, 0, 20); }
static volatile ULONG XeDigestMismatches;
#include "required-functions.inc"
struct ReadResult { NTSTATUS status; ULONG head, tail, references; };
extern "C" ReadResult fixture_section() {
    UCHAR data[16]; std::memset(data, 0xcc, sizeof(data));
    USHORT head=3, tail=5;
    XBE_SECTION_HEADER section{};
    section.VirtualAddress=reinterpret_cast<uintptr_t>(data);
    section.VirtualSize=16; section.SizeOfRawData=8;
    section.HeadSharedPageReferenceCount=&head; section.TailSharedPageReferenceCount=&tail;
    NTSTATUS status=XepLoadSection(nullptr, &section);
    return {status, head, tail, section.SectionReferenceCount};
}
