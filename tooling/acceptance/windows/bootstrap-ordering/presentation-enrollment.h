#pragma once
#include <windows.h>

namespace presentation_enrollment {
constexpr wchar_t Variable[] = L"MIVUBI_EDUCATION_PRESENTATION";
struct Snapshot {
    DWORD bytes, armed;
    LONG factories, candidates, ignored, failures;
    DWORD width, height;
};
// prepare only copies bounded configuration; attach joins the caller's startup
// Detours transaction. Neither creates a factory/device/timer under loader lock.
bool prepare() noexcept;
bool attach() noexcept;
Snapshot snapshot() noexcept;
}
