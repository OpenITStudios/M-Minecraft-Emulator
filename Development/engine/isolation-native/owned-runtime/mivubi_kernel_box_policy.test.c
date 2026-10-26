#include <stddef.h>
#include <wchar.h>
#include "mivubi_kernel_box_policy.h"

/* Always-active checks: CMake Release builds must never remove tests. */
static unsigned checks = 0;
static int require_name(const wchar_t *name, int expected)
{
    ++checks;
    return MivubiBoxNameAllowed(name) == expected;
}

int main(void)
{
    const wchar_t *allowed[] = {
        L"MivubiEdu02", L"MivubiEdu03", L"MivubiEdu04",
        L"mivubiedu02", L"MIVUBIEDU03", L"MiVuBiEdU04"
    };
    const wchar_t *forbidden[] = {
        L"", L"DefaultBox", L"MinecraftClient02", L"MivubiEdu01",
        L"MivubiEdu05", L"MivubiEdu2", L"MivubiEdu002",
        L"MivubiEdu02_Extra", L"MivubiEdu02 ", L" MivubiEdu02",
        L"MivubiEdu02\\", L"MivubiEdu02/Other", L"MivubiEdu02.",
        L"MivubiEdu02:", L"MivubiEdu02\n", L"MivubiEdu03X",
        L"MivubiEdu0", L"MivubiEdu", L"Mivubi", L"Other",
        L"MivubiEdu02\u00e9", L"MivubiEdu04\t", L"MivubiEdu04;",
        L"MivubiEdu04.dll", L"MivubiEdu04x", L"MivubiEdu04_"
    };
    for (size_t i = 0; i < sizeof(allowed) / sizeof(allowed[0]); ++i)
        if (!require_name(allowed[i], 1)) return 1;
    for (size_t i = 0; i < sizeof(forbidden) / sizeof(forbidden[0]); ++i)
        if (!require_name(forbidden[i], 0)) return 2;
    if (!require_name(NULL, 0)) return 3;

    /* Short terminated names must be rejected without reading beyond NUL. */
    const wchar_t empty[1] = {0};
    const wchar_t short_name[2] = {L'M', 0};
    if (!require_name(empty, 0) || !require_name(short_name, 0)) return 4;

    return checks == 35 ? 0 : 5;
}
