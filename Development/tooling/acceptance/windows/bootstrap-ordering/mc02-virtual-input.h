#pragma once
#include "education-config.h"
// Private, compile-opt-in input-state consumer in an already verified game.
// Does not own a process, create IPC, or inject a new DLL.
namespace mc02_input {
bool attach(const education_bootstrap::Config& config) noexcept;
}
