# One virtual Xbox 360 controller — experimental source only

**This is an actual XInput gamepad SIGNAL candidate, NOT independent
Minecraft-client input.** It emits ONE globally visible virtual gamepad
through the third-party ViGEmBus driver. It is not attached to a specific
Minecraft process, profile, HWND, Windows session, or Codex tool.

**Status:** AUTHORED ONLY. NOT BUILT, NOT INSTALLED, NOT EXECUTED.
Not part of the Tauri workspace, installer, Lazy Tester MCP, packaged
binary, DEV.cmd, or any CI. No new Windows driver is added to this repo.

## Why this candidate

Unlike a fake `WM_KEYDOWN` message, a virtual Xbox controller can generate
real analog movement/looking and gamepad button states using Microsoft's
controller input stack. The MIT/Apache-2.0
[`vigem-rust` v0.2](https://github.com/arounre/vigem-rust)
wraps the existing ViGEmBus kernel driver.

A **real controller** does **not** imply:
- MC02 will read it when MC02 is unfocused.
- It is bound to MC02 instead of MC01, MC03, or every Minecraft.
- Minecraft Education uses XInput rather than another input API.
- The same mapping will work with customized gamepad settings.
- Four AI controllers can operate separately, or five AI clients work.
- The human's keyboard/mouse remain unaffected if gamepad signals enter MC01.

The source must NOT be promoted into production input until independent
game-to-device binding and background game acceptance are proven. In
particular, **never run this while the human is playing MC01**, and never
install ViGEmBus on the user's PC without explicit approval.

## Guarded lab interface (do NOT execute yet)

When an operator explicitly authorizes real Windows experiments later,
one lab executable could be compiled separately and launched with both:

```text
LAZY_TESTER_GLOBAL_GAMEPAD_LAB=I_ACKNOWLEDGE_NOT_PER_MINECRAFT
lazy-tester-global-gamepad-lab.exe --unassigned-global-pad-lab
```

This is **not permission or an instruction to run it now**.

The executable uses a local bounded stdin JSON-line protocol (one record
at most 512 bytes, strictly increasing sequence and report time) and controls **one**
virtual controller at most ten seconds. Example record:

```json
{"seq":1,"atMs":0,"lx":0,"ly":32767,"rx":0,"ry":0,"lt":0,"rt":0,"buttons":["a"]}
```

`lx/ly` = left stick, `rx/ry` = right stick, `lt/rt` = triggers;
allowed face/shoulder buttons are `a,b,x,y,left_shoulder,right_shoulder`.
Input reports include a monotonic `atMs` relative timestamp (0–10000 ms),
so queued reports can be replayed with bounded timing. The associated
pure source candidate in `../global-gamepad-candidate.mjs` emits periodic
200 ms updates and one final neutral report; it is not connected to MCP.

Each signal resets to neutral after 350 ms without a new input frame,
on normal stop/EOF and when the ten-second lab window ends.
The virtual pad disconnects when the process exits. Neither timeout
nor device disconnect is accepted as proof of Minecraft releasing input.

The complete target for Lazy Tester is still **one human + four AI**
or **five AI** on five genuine Education clients, all in one P2P world.
One global virtual controller cannot meet that objective alone.

## Future proof requirement

1. Confirm Minecraft Education (exact release) accepts XInput.
2. Verify a single controller reaches MC02 **without MC01 receiving it**,
   including when MC02 is backgrounded.
3. Prove separate controller-to-process binding for MC02–MC05 in the same
   Windows session (likely requires hooking/isolated session support).
4. Verify actual game movement, camera and attack, measured with visual
   frames and attributed Skills Probe telemetry, not driver ACK.
5. Require independent watchdog/release-all and never automate game DLL
   injection, signed-driver install, focus changes or desktop takeover.

Sources:
- https://github.com/arounre/vigem-rust
- https://github.com/nefarius/ViGEmBus
- https://learn.microsoft.com/en-us/gaming/gdk/docs/reference/input/gameinput/interfaces/igameinput/methods/igameinput_setfocuspolicy
- https://github.com/SplitScreen-Me/splitscreenme-protoInput

**All tests/builds/gameplay remain deferred until explicit operator approval.**
