#include "receipt.h"

extern "C" __declspec(dllexport) int DependencyCanary() { return 7; }

extern "C" BOOL WINAPI DllMain(HINSTANCE, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        probe::canary(probe::Dependency);
        probe::registryCanary(probe::Dependency);
    }
    return TRUE;
}
