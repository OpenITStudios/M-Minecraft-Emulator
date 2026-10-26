/* Portable native test for the SAME C definitions in forked driver/Start. */
#include <wchar.h>
#include <stddef.h>
#include "mivubi_core_identity.h"

_Static_assert(sizeof(MIVUBI_PRIVATE_DRIVER_ID) / sizeof(wchar_t) <= 16,
               "Sandboxie API_GET_VERSION WCHAR[16] would overflow");
_Static_assert(MIVUBI_PRIVATE_DRIVER_ABI == 0x57230U,
               "Pinned Sandboxie driver ABI changed unexpectedly");

int main(void) {
    const wchar_t *official = L"5.73.6";
    const wchar_t *private_driver = MIVUBI_PRIVATE_DRIVER_ID;
    wchar_t observed[16] = {0};
    if (wcscmp(private_driver, official) == 0) return 1;
    if (wcscmp(private_driver, L"5.73.6-MIVU") != 0) return 2;
    if (wcslen(private_driver) >= sizeof(observed) / sizeof(observed[0])) return 3;
    for (size_t i = 0; i < sizeof(MIVUBI_PRIVATE_DRIVER_ID) / sizeof(wchar_t); ++i)
        observed[i] = private_driver[i];
    if (wcscmp(observed, MIVUBI_PRIVATE_DRIVER_ID) != 0) return 4;
    if (!MivubiOwnedDriverMatches(observed, 16, MIVUBI_PRIVATE_DRIVER_ABI)) return 5;
    if (MivubiOwnedDriverMatches(official, 16, MIVUBI_PRIVATE_DRIVER_ABI)) return 6;
    if (MivubiOwnedDriverMatches(observed, 16, MIVUBI_PRIVATE_DRIVER_ABI + 1)) return 7;
    if (MivubiOwnedDriverMatches(NULL, 16, MIVUBI_PRIVATE_DRIVER_ABI)) return 8;
    if (MivubiOwnedDriverMatches(observed, 0, MIVUBI_PRIVATE_DRIVER_ABI)) return 9;
    if (MivubiOwnedDriverMatches(observed, 4, MIVUBI_PRIVATE_DRIVER_ABI)) return 10;
    /* Minimal unterminated response: a previous wcscmp would over-read. */
    const wchar_t short_response[2] = { L'5', L'.' };
    if (MivubiOwnedDriverMatches(short_response, 2, MIVUBI_PRIVATE_DRIVER_ABI)) return 15;
    /* A foreign kernel may provide WCHAR[16] without NUL; never scan past it. */
    for (size_t i = 0; i < 16; ++i) observed[i] = L'Z';
    if (MivubiOwnedDriverMatches(observed, 16, MIVUBI_PRIVATE_DRIVER_ABI)) return 11;
    for (size_t i = 0; i < sizeof(MIVUBI_PRIVATE_DRIVER_ID) / sizeof(wchar_t); ++i)
        observed[i] = MIVUBI_PRIVATE_DRIVER_ID[i];
    if (!MivubiOwnedDriverMatches(observed, 16, MIVUBI_PRIVATE_DRIVER_ABI)) return 12;
    /* Even with a matching prefix, the terminator must fit in the given size. */
    if (MivubiOwnedDriverMatches(observed, sizeof(MIVUBI_PRIVATE_DRIVER_ID) /
        sizeof(wchar_t) - 1, MIVUBI_PRIVATE_DRIVER_ABI)) return 13;
    observed[0] = L'X';
    if (MivubiOwnedDriverMatches(observed, 16, MIVUBI_PRIVATE_DRIVER_ABI)) return 14;
    return 0;
}
