/* MIVUBI-owned Start.exe profile admission. Portable, dependency-free C++17.
 * This source is inlined into the reviewed upstream Start.cpp fork patch.
 * It does not authenticate Microsoft signatures, attest a driver, or create boxes.
 */
struct MivubiOwnedLaunch {
    const wchar_t* box;
    wchar_t* command;
};

enum class MivubiOwnedDecision { NotOwned, Rejected, Admitted };

static bool MivubiAsciiEqual(wchar_t a, wchar_t b) {
    if (a >= L'A' && a <= L'Z') a += (L'a' - L'A');
    if (b >= L'A' && b <= L'Z') b += (L'a' - L'A');
    return a == b;
}

static bool MivubiAsciiName(const wchar_t* begin, const wchar_t* end,
                            const wchar_t* expected) {
    while (begin != end && *expected) {
        if (!MivubiAsciiEqual(*begin++, *expected++)) return false;
    }
    return begin == end && *expected == L'\0';
}

static MivubiOwnedDecision MivubiParseOwnedStart(wchar_t* cmd,
                                                MivubiOwnedLaunch* result) {
    if (!cmd || !result) return MivubiOwnedDecision::Rejected;
    static const wchar_t prefix[] = L"/mivubi_client:";
    for (unsigned i = 0; i < sizeof(prefix) / sizeof(prefix[0]) - 1; ++i) {
        if (cmd[i] == L'\0' || cmd[i] != prefix[i])
            return MivubiOwnedDecision::NotOwned;
    }
    wchar_t* p = cmd + (sizeof(prefix) / sizeof(prefix[0]) - 1);
    // Examine one character at a time. Short-circuit at the first NUL:
    // a truncated command must not read p[1], p[2] or p[3] past its buffer.
    if (p[0] != L'0') return MivubiOwnedDecision::Rejected;
    if (p[1] != L'2' && p[1] != L'3' && p[1] != L'4')
        return MivubiOwnedDecision::Rejected;
    if (p[2] != L' ' || p[3] != L'"') return MivubiOwnedDecision::Rejected;
    const wchar_t* box = p[1] == L'2' ? L"MivubiEdu02" :
                         p[1] == L'3' ? L"MivubiEdu03" : L"MivubiEdu04";
    wchar_t* command = p + 3;
    const wchar_t* file = command + 1;
    const wchar_t* end = file;
    unsigned length = 0;
    while (*end && *end != L'"' && length <= 1024) { ++end; ++length; }
    if (*end != L'"' || end[1] != L'\0' || length < 4 || length > 1024 ||
        !((file[0] >= L'A' && file[0] <= L'Z') ||
          (file[0] >= L'a' && file[0] <= L'z')) ||
        file[1] != L':' || file[2] != L'\\') return MivubiOwnedDecision::Rejected;
    const wchar_t* component = file + 3;
    const wchar_t* basename = component;
    for (const wchar_t* it = component; it != end; ++it) {
        wchar_t c = *it;
        if (c < L' ' || c == L'\x7f' || c == L'/' || c == L':' ||
            c == L'<' || c == L'>' || c == L'|' || c == L'*' ||
            c == L'?' || c == L'"') return MivubiOwnedDecision::Rejected;
        if (c == L'\\') {
            if (it == component || it[-1] == L'\\' ||
                (it - component == 1 && component[0] == L'.') ||
                (it - component == 2 && component[0] == L'.' && component[1] == L'.'))
                return MivubiOwnedDecision::Rejected;
            component = it + 1;
            basename = component;
        }
    }
    if (basename == end ||
        !((MivubiAsciiName(basename, end, L"Minecraft.Windows.exe")) ||
          (MivubiAsciiName(basename, end, L"MinecraftEducation.exe")) ||
          (MivubiAsciiName(basename, end, L"MinecraftEducationEdition.exe"))))
        return MivubiOwnedDecision::Rejected;
    result->box = box;
    result->command = command;
    return MivubiOwnedDecision::Admitted;
}
