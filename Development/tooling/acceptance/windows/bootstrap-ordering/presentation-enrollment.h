#pragma once
#include <windows.h>
#include <cwchar>
#include <cstddef>
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
    HANDLE hudSelected; // v4 only: independent manual-reset HUD selection.
};
constexpr SIZE_T LegacyPolicyBytes = 120;
static_assert(offsetof(PolicyConfig, hudSelected) == LegacyPolicyBytes);
static_assert(sizeof(PolicyConfig) == 128);
inline bool pacingLimits(const PolicyConfig& config, unsigned& active, unsigned& background) noexcept {
    return lazytester_presentation_limits(config.creation.version, config.reserved, &active, &background) != 0;
}
// Metadata only: never create a factory/device while the loader lock is held.
inline bool imageMetadata(CreationConfig& result, HMODULE& module) noexcept {
    LazyTesterPresentationImage image{}; HMODULE loaded=nullptr;
    if (!lazytester_presentation_image(&image,&loaded)) return false;
    result.imageTimestamp=image.timestamp; result.imageBytes=image.bytes; result.imageId=image.id;
    module=loaded; return true;
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
