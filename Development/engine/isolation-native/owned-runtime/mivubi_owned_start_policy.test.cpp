#include <cstdlib>
#include <cwchar>
#include <string>
#include "mivubi_owned_start_policy.h"

using Decision = MivubiOwnedDecision;
static unsigned checks = 0;
static void require(bool condition) { if (!condition) std::abort(); }

static void check(const wchar_t* raw, Decision expected,
                  const wchar_t* box = nullptr) {
    std::wstring command(raw);
    MivubiOwnedLaunch result{nullptr, nullptr};
    const Decision actual = MivubiParseOwnedStart(command.data(), &result);
    require(actual == expected);
    if (actual == Decision::Admitted) {
        require(result.box && result.command);
        require(box && std::wcscmp(box, result.box) == 0);
        require(result.command[0] == L'"');
    } else {
        require(!result.box && !result.command);
    }
    ++checks;
}

int main() {
    check(L"/box:MivubiEdu02 \"C:\\Minecraft.Windows.exe\"", Decision::NotOwned);
    check(L"/mivubi_client:02 \"C:\\Program Files\\Minecraft.Windows.exe\"",
          Decision::Admitted, L"MivubiEdu02");
    check(L"/mivubi_client:03 \"D:\\Games\\MinecraftEducation.exe\"",
          Decision::Admitted, L"MivubiEdu03");
    check(L"/mivubi_client:04 \"E:\\Test\\MinecraftEducationEdition.exe\"",
          Decision::Admitted, L"MivubiEdu04");
    check(L"/mivubi_client:02 \"C:\\MINECRAFT.WINDOWS.EXE\"",
          Decision::Admitted, L"MivubiEdu02");
    check(L"/mivubi_client:01 \"C:\\Minecraft.Windows.exe\"", Decision::Rejected);
    check(L"/mivubi_client:05 \"C:\\Minecraft.Windows.exe\"", Decision::Rejected);
    check(L"/mivubi_client:02 C:\\Minecraft.Windows.exe", Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\Minecraft.Windows.exe\" /terminate_all",
          Decision::Rejected);
    check(L"/mivubi_client:02 \"\\\\server\\Minecraft.Windows.exe\"", Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\..\\Minecraft.Windows.exe\"", Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\folder\\..\\Minecraft.Windows.exe\"",
          Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\Notepad.exe\"", Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\folder\\\\Minecraft.Windows.exe\"",
          Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\Minecraft.Windows.exe\\\"", Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\Minecraft.Windows.exe\"\"", Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\folder\\.\\Minecraft.Windows.exe\"",
          Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\folder\\Minecraft.Windows.exe:ads\"",
          Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\folder:stream\\Minecraft.Windows.exe\"",
          Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\folder/other\\Minecraft.Windows.exe\"",
          Decision::Rejected);
    check(L"/mivubi_client:02 \"C:/Minecraft.Windows.exe\"", Decision::Rejected);
    check(L"/mivubi_client:02 \"C:\\Minecraft.Windows.exe\"", Decision::Admitted,
          L"MivubiEdu02");
    // Precisely truncated buffers are important: a parser that reads
    // p[2]/p[3] before detecting NUL will overrun these heap strings.
    check(L"/mivubi_client:", Decision::Rejected);
    check(L"/mivubi_client:0", Decision::Rejected);
    check(L"/mivubi_client:02", Decision::Rejected);
    check(L"/mivubi_client:02 ", Decision::Rejected);
    check(L"/mivubi_client:02 \"", Decision::Rejected);
    check(L"/mivubi_clien", Decision::NotOwned);
    // A Release build defines NDEBUG. Never rely on disabled assert().
    return checks >= 26 ? 0 : 1;
}
