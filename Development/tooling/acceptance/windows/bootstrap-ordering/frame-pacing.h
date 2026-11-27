#pragma once
#include <windows.h>
#include "frame-pacing-ffi.h"
#include <stdexcept>

// One rendering consumer, policy changes from another thread; join before destruction.
// No game hook, simulation scheduling, process throttling or global timer change.
namespace presentation_pacing {
enum class Outcome { Frame, Stopped, Error };
struct Permit {
    Outcome outcome;
    unsigned ceiling;
    LONGLONG released;
};

// Trusted producer contract: manual-reset level/stop, auto-reset changed, and
// the controller PROCESS handle. The consumer duplicates SYNCHRONIZE rights;
// the capability decoder must establish object types/identity before handing off.
inline bool validCeilings(unsigned active, unsigned background) noexcept {
    return lazytester_frame_pacing_valid_ceilings(active, background) != 0;
}
struct Input {
    HANDLE level = nullptr, changed = nullptr, stop = nullptr, controller = nullptr;
    unsigned activeFps = 144, backgroundFps = 10;
    HANDLE hudSelected = nullptr; // HUD only; never consumed by Pacer.
};
// Thin native compatibility adapter: Rust owns the entire pacing state,
// waitable timer, duplicated handles, deadline and sticky stop.
class Pacer {
    LazyTesterNativePacer* state_;
public:
    explicit Pacer(const Input& input = {}) : state_(lazytester_pacer_create(
        input.level, input.changed, input.stop, input.controller, input.activeFps, input.backgroundFps)) {
        if (!state_) throw std::runtime_error("Pacing capability unavailable");
    }
    Pacer(const Pacer&) = delete;
    Pacer& operator=(const Pacer&) = delete;
    ~Pacer() { lazytester_pacer_destroy(state_); }
    bool foreground(bool active) { return lazytester_pacer_foreground(state_, active ? 1U : 0U) != 0; }
    bool stop() { return lazytester_pacer_stop(state_) != 0; }
    Permit wait_for_frame() {
        LazyTesterFramePermit result{};
        lazytester_pacer_wait(state_, &result);
        return {result.outcome == 0 ? Outcome::Frame : result.outcome == 1 ? Outcome::Stopped : Outcome::Error,
            result.ceiling, result.released};
    }
};
} // namespace presentation_pacing
