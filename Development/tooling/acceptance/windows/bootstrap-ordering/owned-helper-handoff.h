#pragma once
#include <windows.h>
#include <cstddef>

// Own-fixture transport only, never a game config or public provider interface.
namespace owned_helper {
constexpr DWORD Magic = 0x484c5031;
enum Decision : LONG { Pending, Accepted, Refused };
struct Handoff {
    DWORD magic, version, bytes, reserved;
    HANDLE configuration, proof, ready, reply;
    HANDLE process, thread; // Values in the original sender, not receiver handles or PIDs.
    DWORD pid, error;
    FILETIME created;
    volatile LONG decision;
    DWORD reservedEnd;
};
static_assert(sizeof(Handoff) == 88);
static_assert(offsetof(Handoff, process) == 48);
static_assert(offsetof(Handoff, created) == 72);
static_assert(offsetof(Handoff, decision) == 80);
}
