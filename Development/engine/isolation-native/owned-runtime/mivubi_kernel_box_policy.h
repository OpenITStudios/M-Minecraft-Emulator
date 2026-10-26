/* MIVUBI-owned kernel sandbox namespace for the exclusive private fork.
 * This exact C implementation is injected into Sandboxie/core/drv/box.c.
 * Reject unknown boxes BEFORE allocating any kernel BOX object; cloned
 * descendant contexts pass through that same allocation entrypoint.
 * It is a product namespace gate, not a replacement for Sandboxie hooks.
 */
#ifndef MIVUBI_KERNEL_BOX_POLICY_H
#define MIVUBI_KERNEL_BOX_POLICY_H

static int MivubiBoxAsciiEqual(wchar_t lhs, wchar_t rhs)
{
    if (lhs >= L'A' && lhs <= L'Z') lhs += L'a' - L'A';
    if (rhs >= L'A' && rhs <= L'Z') rhs += L'a' - L'A';
    return lhs == rhs;
}

static int MivubiBoxNameAllowed(const wchar_t *name)
{
    static const wchar_t prefix[] = L"MivubiEdu0";
    unsigned i;
    if (name == 0) return 0;
    for (i = 0; i < sizeof(prefix) / sizeof(prefix[0]) - 1; ++i) {
        if (!name[i] || !MivubiBoxAsciiEqual(name[i], prefix[i]))
            return 0;
    }
    return (name[i] == L'2' || name[i] == L'3' || name[i] == L'4')
        && name[i + 1] == L'\0';
}
#endif
