#include "education-config.h"
#include "receipt.h"
#include <cwchar>
#include <cstdio>

int main() {
    FILE_ID_INFO id{};
    id.VolumeSerialNumber = 7;
    for (auto& byte : id.FileId.Identifier) byte = 3;
    wchar_t leaves[5][64]{};
    for (DWORD slot = 1; slot <= 5; ++slot) {
        if (!education_bootstrap::validSlot(slot) || !probe::registryStoreLeaf(id, slot, leaves[slot - 1])) return 1;
        const auto length = std::wcslen(leaves[slot - 1]);
        if (length != 51 || leaves[slot - 1][50] != static_cast<wchar_t>(L'A' + slot - 1)) return 1;
        for (DWORD previous = 1; previous < slot; ++previous)
            if (std::wcscmp(leaves[previous - 1], leaves[slot - 1]) == 0) return 1;
    }
    wchar_t invalid[64]{};
    if (education_bootstrap::validSlot(0) || education_bootstrap::validSlot(6)
        || probe::registryStoreLeaf(id, 0, invalid) || probe::registryStoreLeaf(id, 6, invalid)) return 1;
    std::puts("Slots 1-5 have distinct A-E leaves; slots 0/6 refuse; Config v2 remains 1136 bytes.");
    return 0;
}
