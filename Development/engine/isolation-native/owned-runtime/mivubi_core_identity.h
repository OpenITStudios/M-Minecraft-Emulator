/* Shared identity for the MIVUBI-owned Sandboxie-derived core.
 * Included verbatim in the pinned private driver API and Start.exe sources.
 * This is runtime identification, NOT signing/attestation or a new ABI.
 */
#ifndef MIVUBI_OWNED_CORE_IDENTITY_H
#define MIVUBI_OWNED_CORE_IDENTITY_H
#define MIVUBI_PRIVATE_DRIVER_ID L"5.73.6-MIVU"
#define MIVUBI_PRIVATE_DRIVER_ABI 0x57230U
#include <stddef.h>
#include <wchar.h>
/* API_GET_VERSION supplies at most WCHAR[16], possibly without a terminator
 * when the device is malformed or foreign. Never run unbounded wcscmp over
 * a driver-provided buffer. Compare the terminator within the caller's
 * explicitly provided WCHAR capacity, with no CRT string reads.
 */
static inline int MivubiOwnedDriverMatches(const wchar_t *observed,
                                           size_t available_wchars,
                                           unsigned long abi) {
    static const wchar_t required[] = MIVUBI_PRIVATE_DRIVER_ID;
    const size_t required_wchars = sizeof(required) / sizeof(required[0]);
    if (observed == NULL || abi != MIVUBI_PRIVATE_DRIVER_ABI ||
        available_wchars < required_wchars) return 0;
    for (size_t i = 0; i < required_wchars; ++i) {
        if (observed[i] != required[i]) return 0;
    }
    return 1;
}
/* API_GET_VERSION accepts a WCHAR[16]. The marker must fit, including NUL. */
#endif
