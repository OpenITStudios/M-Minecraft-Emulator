#pragma once
#include <windows.h>
#include <cwchar>
#include "education-config.h"
#include "frame-pacing.h"

namespace presentation_enrollment {
constexpr wchar_t Variable[] = L"MIVUBI_EDUCATION_PRESENTATION";
constexpr wchar_t SectionVariable[] = L"MIVUBI_EDUCATION_PRESENTATION_SECTION";
// Separate, read-only, process-bound capability; the storage ABI stays v2.
struct CreationConfig {
    DWORD magic, version, bytes, pid;
    FILETIME created;
    DWORD session, imageTimestamp, imageBytes, oldRva, hwndRva;
    FILE_ID_INFO imageId;
};
constexpr DWORD CreationMagic = 0x50524431;
static_assert(sizeof(CreationConfig) == 72);
// Private v2 extension; observe v1 and the storage configuration stay unchanged.
struct PolicyConfig {
    CreationConfig creation;
    HANDLE level, changed, stop, controller;
    DWORD controllerPid, reserved; // v2: zero; v3: active FPS low 16 bits, background high 16 bits.
    FILETIME controllerCreated;
};
static_assert(sizeof(PolicyConfig) == 120);
inline bool pacingLimits(const PolicyConfig& config, unsigned& active, unsigned& background) noexcept {
    if (config.creation.version == 2 && config.reserved == 0) { active=144; background=10; return true; }
    if (config.creation.version != 3) return false;
    active=config.reserved & 0xffff; background=config.reserved >> 16;
    return presentation_pacing::validCeilings(active,background);
}
// Metadata only: never create a factory/device while the loader lock is held.
inline bool imageMetadata(CreationConfig& result, HMODULE& module) noexcept {
    module = GetModuleHandleW(L"dxgi.dll");
    constexpr DWORD capacity = 1024;
    wchar_t actual[capacity]{}, system[capacity]{};
    const DWORD moduleLength = module ? GetModuleFileNameW(module, actual, capacity) : 0;
    if (!moduleLength || moduleLength >= capacity) return false;
    const UINT length = GetSystemDirectoryW(system, capacity);
    if (!length || length + 10 >= capacity || wcscat_s(system, L"\\dxgi.dll")
        || _wcsicmp(actual, system)) return false;
    HANDLE file = CreateFileW(system, FILE_READ_ATTRIBUTES, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        nullptr, OPEN_EXISTING, FILE_FLAG_OPEN_REPARSE_POINT, nullptr);
    if (file == INVALID_HANDLE_VALUE) return false;
    FILE_ATTRIBUTE_TAG_INFO attributes{};
    const bool valid = GetFileInformationByHandleEx(file, FileAttributeTagInfo, &attributes, sizeof(attributes))
        && !(attributes.FileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT))
        && GetFileInformationByHandleEx(file, FileIdInfo, &result.imageId, sizeof(result.imageId));
    CloseHandle(file);
    if (!valid) return false;
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(module);
    if (dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew <= 0 || dos->e_lfanew > 4096) return false;
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(reinterpret_cast<const BYTE*>(module) + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE || nt->FileHeader.Machine != IMAGE_FILE_MACHINE_AMD64
        || nt->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC) return false;
    result.imageTimestamp = nt->FileHeader.TimeDateStamp;
    result.imageBytes = nt->OptionalHeader.SizeOfImage;
    return result.imageBytes > 4096;
}
struct Snapshot {
    DWORD bytes, armed;
    LONG factories, candidates, ignored, failures;
    DWORD width, height;
};
// prepare validates a bounded, copied capability; attach joins the caller's startup
// Detours transaction. Neither creates a factory/device/timer under loader lock.
bool prepare(const education_bootstrap::Config& process) noexcept;
bool attach() noexcept;
Snapshot snapshot() noexcept;
}
