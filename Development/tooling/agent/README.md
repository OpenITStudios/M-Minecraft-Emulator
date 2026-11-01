# Windows AI test bridge — source implementation

The bridge exists for **Minecraft Education peer-to-peer client testing** using the five native profiles owned by Lazy Tester. The ordinary bridge does NOT start a dedicated server, create a second game controller or modify account/world data. Optional compile-OFF native per-client input experiments are source-only candidates, NOT tested Minecraft gameplay.

## 1 November 2026: 4/5 AI input source candidate

The original MC02 virtual polling candidate has been generalized for four AI clients
(MC02–MC05 in human-plus-four-ai mode) or five clients (MC01–MC05 in five-ai mode).
Each explicitly opted-in game process receives its OWN inherited 96-byte read-only
input mapping; one retained authenticated native controller remains the only writer.
Hybrid requires real human MC01 foreground for each active AI packet and forbids
AI input on MC01. Five-ai requires its own explicit per-slot AI launch and never
uses global SendInput, foreground takeover or a second game owner.

Experimental native code is OFF by default, excluded from ordinary packaging,
and NOT authorized for execution. Preferred source build flags are Cargo feature
multi-client-native-input (alias of historical mc02-native-input) and test-only
CMake option BOOTSTRAP_MULTI_CLIENT_VIRTUAL_INPUT. The old private
MIVUBI_EDUCATION_MC02_INPUT_SECTION name and Mc02Input request remain unchanged
for source compatibility; both are now bound to an exact selected original slot.

For a separately approved developer setup ONLY: the MCP opt-in option is
--multi-client-input-experimental on with --mode human-plus-four-ai or five-ai,
and the required environment consent is:

    LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT=I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT

The extra experimental tools are run_experimental_multi_client_actions and
stop_experimental_multi_client_actions. The first compiles up to 32 semantic
actions into at most 64 input pulses per client within 3.5 seconds and
dispatches one same-image bounded CLI schedule for the requested AI slots.
The second cancels the WHOLE active cross-slot sequence (not one slot from a
shared schedule), or requests independent neutral states on named idle slots.
The native 350ms TTL is only a last-resort state expiry, not gameplay evidence.

This implementation does not prove Minecraft Education consumes keyboard,
Raw Input, relative mouse, attack/use, or state releases. It also does NOT
prove that MC01 remains unaffected, host/join succeeds, screenshots are fresh,
or four/five game clients can run AI movement simultaneously. Node/Rust/MSVC
tests, Windows build, packaging, Minecraft and user-PC execution remain
NOT_EXECUTED. Original MC02-only tools are retained only as a legacy hybrid
experiment and cannot be enabled simultaneously with the multi-client flag.

## Coherent timestamp-group native scheduling — source only

The optional multi-client native scheduler now sends all AI pulses
sharing one timestamp in a SINGLE authenticated `AiInputGroup`
request on the existing controller pipe, avoiding 4–5 independent
five-profile IPC receipts for that timestamp. Up to five distinct
client slots are allowed, in ascending order; each pulse retains
its independent keys, relative mouse deltas and original process
channel. A group is **not** atomic or guaranteed simultaneous:
the single retained actor still validates and writes slots in
sequence.

The originating CLI computes the monotonic deadline at each due
timestamp +150 ms. The controller checks that deadline before
every game-local channel write. Late delivery refuses the group.
If a member fails, the controller attempts neutral ReleaseAll for
each possibly touched member of that group. Independently, the
CLI retains its original release pass across every attempted
slot in all groups, followed by the MCP neutral cleanup and
native TTL fallback if a process is interrupted. Partial or
uncertain release must NOT report success.

`timestampGroups` in the CLI receipt is checked against the MCP
submitted schedule and surfaced as
`timestampGroupsAcknowledged`. The unchanged
`PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF` marker is
transport-only; runtime input consumption and latency are NOT
EXECUTED or verified. This changes only the separately opted-in
native input route; default builds, normal manager, legacy
MC02-only input and user data remain unchanged.

## Coordinated 4–5 AI visual decision loop (source-only)

The existing Visual Observer owns temporary, bounded frame caches,
and the original per-client Cua/WGC capture path already validates
native PID/HWND/process birth/session before and after capture.
There is no second observer, model, or autonomous background loop.

1. Start the existing `start_visual_observation` for the AI-owned slots.
   In hybrid mode these are MC02–MC05; five-AI can use MC01–MC05.
2. Call `get_multi_client_visual_state` (no arguments defaults to the
   mode's AI slots). It returns one compact metadata-only cohort:
   per-slot sampled count, changed-byte sequence, frame freshness,
   exact capture-process binding and an optional fresh `baseline`.
   No PNG bytes are sent by this tool, and no new capture is triggered.
3. Call the existing `read_visual_observation` for EACH necessary slot
   to send its cached PNG to Codex/Claude. Model reasoning/decision
   happens outside Lazy Tester; metadata alone is not visual reasoning.
4. Under separately approved experimental multi-client opt-in ONLY,
   submit `run_experimental_observed_actions` with at most 32 bounded
   semantic actions and a `basis` array containing the exact
   `frames[].baseline` from the preceding team-state result for every
   action slot. Changed/missing/aged-out images or replaced game
   windows are rejected before native dispatch. The original one-CLI
   multi-slot scheduler and retained game-owner validation are reused.
5. Follow the returned `nextObservation` instruction with another
   `get_multi_client_visual_state`, then request individual PNGs
   again. `sampledSinceBaseline` means a sampler delivered a newer
   capture. `pngBytesChangedSinceBaseline` means encoded PNG bytes
   changed. Neither means Minecraft accepted WASD, changed view, or
   joined the same world.

The original `run_experimental_multi_client_actions` remains available
for explicitly opted-in experiments that do not require image baselines.
No execution tool is advertised on the default MCP. Stale observed
actions cannot silently fall back to an unobserved dispatch.
The team-state tool works without a configured provider, but correctly
reports NOT_OBSERVING/incomplete until real captures exist.

**Proof ceiling:** SOURCE AUTHORED / NOT_EXECUTED. Windows capture,
real AI image reasoning, native input and simultaneous gameplay remain
unverified. Standard installer, user data, fixed slot ownership,
human MC01 isolation and CI policy are unchanged.

## Safe reconnection of the opt-in 4/5-AI MCP (source only)

A newly created MCP process cannot inherit the in-memory input
uncertainty marker of an earlier process. Therefore the explicit
`--multi-client-input-experimental on` configuration starts in
**fail-closed reconciliation** for every AI-owned slot. No input,
game launch, controller IPC, process/window scan or account action
happens automatically on startup. The default read-only MCP remains
unchanged.

`get_multi_client_visual_state` exposes
`coordination.pendingReconnectSlots` and the per-client reason
`NEW_MCP_CONNECTION_REQUIRES_NEUTRAL_OR_FRESH_AI_LAUNCH`.
A fresh visual frame is NOT permission to bypass the hold.

For an existing live opted-in game, explicitly request
`stop_experimental_multi_client_actions({slots:[...]})` to send
the original exact-owner neutral command through the retained
controller; a fresh transport ACK clears only the specified slots.
If the game has no opted-in input channel, the native stop refuses:
do not invent one. Follow the normal game shutdown process, then
explicitly launch the profile under the multi-AI experiment.

For a genuinely **new** profile, `launch_profile({slot})`
carries `launch-ai-hybrid` or `launch-ai-five` to the existing
retained controller. Only a valid successful native launch receipt
showing the selected child STARTING/RUNNING without attention can
clear that slot's reconnect hold. An old window, ordinary Inspect,
failed launch or stopped/attention reply cannot do it. A replacement
MCP process starts with the entire hold again; no durable credential,
recovery database or silent "all safe" flag is written.

Previous aborted/uncertain native-input holds continue to need
explicit neutral ACK. Cross-client AI action batches, WGC/Cua
capture, source timestamps, per-slot ReleaseAll, game identity,
MC01-human foreground checks, opt-in gates and native TTL retain
their existing source contracts. Input ACK does NOT prove that
Minecraft released a key or moved a player. Source tests and Windows
gameplay remain NOT_EXECUTED / NOT_VERIFIED.

## Coordinated 4–5 AI source-loss and uncertain-input containment

`get_multi_client_visual_state` now also returns a read-only
`coordination` projection for each selected AI-owned slot.
`readySlots` means only a usable recent observed source/time;
`blockedSlots` lists missing/stale frames, source timing problems,
changed native window identity since the supplied baseline, or
`UNCERTAIN_NATIVE_INPUT_REQUIRES_EXPLICIT_NEUTRAL`. No input is
dispatched by this read. MC01 is always excluded in hybrid mode.

The existing `run_experimental_observed_actions` uses strict
all-selected-slots admission. **It never automatically drops an
unavailable AI and sends actions to the remaining clients.** To
continue with fewer independent AI clients, the model/operator must
explicitly submit a NEW action list for only the chosen healthy
slots and fetch matching fresh per-slot baselines.

With explicit multi-client native opt-in, failed/aborted/uncertain
native sequence ACKs or failed neutral releases put all possibly
touched slots in a local input-recovery hold. Even if automatic
best-effort cleanup ACKs, the next action cannot retry any held slot
without a new operator/AI invocation of
`stop_experimental_multi_client_actions({slots:[...]})`. That idle
stop sends the existing native per-client neutral command, and only
acknowledged exact selected slots leave the local hold. A failed
neutral leaves the hold; unaffected slots are not stopped or silently
reselected. An ACK is a transport state write, not proof game keys
were released.

An observed round is also aborted if one of its selected visual
samplers expires, is stopped, sees a replaced root window, or reaches
a non-busy capture failure. The same global batch cancellation and
per-slot neutral cleanup are reused; no background controller or new
capture loop is installed. A temporary native-busy capture skip keeps
the original observer's bounded retry behavior.

The recovery hold exists in the ONE MCP process memory, not in the
game or on disk; a newly started MCP cannot infer a previous
process's failed requests. The original native TTL and per-process
identity/channel gates remain mandatory last-resort defenses.
Neither the coordinator's `observedActionEligible` nor native
transport ACKs prove Minecraft actually consumed a command. All
execution, timing improvements and gameplay remain NOT_EXECUTED /
NOT_VERIFIED.

## 4/5-AI visual source-clock synchronization (source only)

The optional visual sampler keeps two different times per window:
`collectedAt` is when MCP received/stored its frame, while
`sourceFrameAtMs` is an earlier, provider-specific event bound to
that frame. Its `sourceTimeKind` has exactly two meanings:

- `WGC_READBACK_COMPLETED`: the optional native producer finished
  reading fitted GPU pixels. The original PID/HWND/birth is checked
  again before encoding/transmitting the PNG. The frame may still have come
  from a previously queued game frame.
- `CUA_QUERY_STARTED`: MCP started the explicit read-only Cua window
  screenshot request. Cua does not expose the actual pixel acquisition
  or game-presentation time here. An old-looking game picture can still
  arrive after this request.

Source clock values are NOT Minecraft-render presentation timestamps.
No timeline claim is made when source timing is missing/ambiguous.
MCP returns a source-time anchor and, for WGC, `producerSequence`
on each freshly held frame. Re-reading the same WGC worker sequence
no longer advances the per-client successful sample counter; source
time does not become newer merely because MCP requested the old PNG
again. Clock regressions or aged source frames fail closed.

`get_multi_client_visual_state` now includes source timing in each
`frames[].baseline` and exposes `sourceTimingUsable`. The
already-opted-in `run_experimental_observed_actions` refuses missing
source timing before input dispatch. The follow-up
`evaluate_experimental_multi_client_outcomes` binds the SAME
source identity, timestamp kind, WGC producer sequence and capture
counter, and only treats a later source event as a candidate
post-action visual observation when its source time is AFTER the
action transport receipt's completion time. All findings remain
external model visual reports, not validated in-game results.

The normal read-only `read_visual_observation` can still return
recently received images without source timing for manual review.
Those images cannot authorize an observed-action round. No new
driver, camera hook, game controller, CI or local execution is
introduced; Windows timing quality and gameplay acceptance remain
NOT_EXECUTED / NOT_VERIFIED.

## Per-client outcome classification from observed action rounds (source-only)

The opt-in `run_experimental_observed_actions` response includes its
original `visualBaseline` plus `mode`, `transportStartedAt`,
`transportCompletedAt` and a canonical SHA-256 `actionFingerprint`.
The digest prevents accidental mixing with another submitted action
plan, but it is not a signature, cannot authenticate a copied/fabricated
receipt, and is not proof Minecraft processed input.
The timestamps are caller-visible transport timestamps, not
authenticated Minecraft action timestamps.

After the requested cross-slot action completes, retrieve another
`get_multi_client_visual_state` for the original AI slots. Then
explicitly request the relevant PNGs with `read_visual_observation`
and compare them with the images viewed before dispatch. Submit
`evaluate_experimental_multi_client_outcomes` with:

- `actions`: the SAME bounded action list used in the round
- `receipt`: the result of `run_experimental_observed_actions`
- `after`: one `frames[].baseline` for each requested slot in ascending
  slot order (use null for an unavailable post-frame)
- `reviews` (optional): only intended action categories, with
  `slot`, `category` (movement, camera, attack, interaction),
  `assessment` (VISIBLE_CHANGE_REPORTED, NO_VISIBLE_CHANGE_REPORTED,
  UNDETERMINED), and a brief relevant `cue` for non-uncertain findings

The new tool NEVER sends a new input action or starts capture. It
reads the EXISTING cache metadata and returns one outcome per expected
category/client. Missing, old, replaced or pre-completion images
remain NOT_EVALUABLE. A visible-change report with identical PNG bytes
is flagged as inconsistent, rather than promoted to success. Its
`totals` count model reports, NOT game successes.

**Important:** the submitted receipt and model findings are untrusted
caller claims; exact metadata comparison does not authenticate them.
The MCP cannot prove Codex/Claude reviewed two images. PNG changes
can be unrelated to movement; an attack animation is not proof of a
hit, and an absent animation is not proof an attack failed. No PASS
or FAIL is claimed for Minecraft gameplay. All real execution
and visual providers remain NOT_EXECUTED/NOT_VERIFIED.

## Exactly what works in source

- A local **stdio MCP** server in `mcp.mjs` exposes profile, Computer Use target, optional read-only visual observation, peer planning, and gameplay evidence tools.
- `inspect_profiles`/`launch_profile` delegate to the **same packaged `lazy-tester.exe`** under `--lazy-tester-agent-cli` which calls the existing authenticated native session controller. A returned `RUNNING` process observation does not prove the game is logged in, has joined a world, or accepts input.
- Evidence tools read completed `[ANALYTICS]` and `[GAME_RESULTS]` JSON lines from the existing `minecraft-bedrock-agent-skills` debug capture script's per-client output. No new debugger listener or protocol is implemented. File-to-MC slot mapping is **manual and unverified** until connected to an observed player/world identity.
- Two modes: `human-plus-four-ai` reserves MC01 for the human host (default); `five-ai` permits MC01 launch as an AI-controlled slot. MCP tools do not give the AI any new right to interact with the protected human's keyboard or window.

**Important:** This increment implements the *control-plane and evidence reading*, **not** parallel gameplay. Real independent WASD/mouse for MC02–MC05 while a person plays MC01 remains `NOT_IMPLEMENTED / NOT_VERIFIED` on Windows. The tools do not claim to host/join a world, send chat commands, or perform simultaneous Minecraft input. Claude/Codex computer use alone does not guarantee four independent background input streams. A separate, verified input backend is required before advertising those capabilities.

## Start for source development (Windows)

The installed app provides `App/lazy-tester.exe` after packaging. The operator must install/prepare Lazy Tester normally; **no account, world or game launches simply by starting the MCP server**. A local Node 24+ installation is required to run the current MCP script. The source-repository path of this script must remain accessible; it is **not yet part of the public three-file installer export**.

Example Codex/Claude MCP server configuration (replace paths with absolute local paths):

```json
{
  "command": "node",
  "args": [
    "C:\\projects\\M-Minecraft-Emulator\\Development\\tooling\\agent\\mcp.mjs",
    "--mode", "human-plus-four-ai",
    "--exe", "C:\\projects\\M-Minecraft-Emulator\\App\\lazy-tester.exe",
    "--evidence-dir", "C:\\projects\\M-Minecraft-Emulator\\workspace\\private\\agent-evidence"
  ]
}
```

Use `--mode five-ai` only when **all five clients are intentionally AI-operated**. `launch_profile` is a state-changing request; authorize it through your agent tool permission UI. The native controller retains its own admission gate and lifecycle checks.

`--lazy-tester-agent-cli inspect` refuses without a compatible retained controller; it never invents an empty five-client state. `launch` uses the reviewed original bootstrap to request a client launch, without silently opening or joining a Minecraft world.

## Synchronized per-client input frames — source-only

`plan_player_actions({actions,includeClientFrames:true})` now compiles
the canonical action timeline into per-slot full button states and
relative mouse deltas grouped by common timestamps, including MC02–MC05.
`includeClientFrames` and `includeTimeline` cannot both be true.

The pure compiler preserves release-before-press ordering, prevents
cross-slot button-state sharing, protects human-owned MC01 in hybrid
mode, and reports required per-slot emergency release. The same
compiler now supplies frames to the existing **MC02-only** experimental
native action runner. Same-timestamp release/repress of one button is
explicitly flagged; the MC02 single-state writer refuses it rather
than silently dropping a fresh press edge.

The `five-ai` mode may plan MC01 as AI, but it does NOT enable actual
MC01 input. MC03–MC05 still lack an input actuator. All frame batches
report `NOT_DISPATCHED`; frames at the same timestamp are coordination
intent, not proven simultaneous Windows/game input. No driver,
additional hook, process controller, CI or product input activation
is introduced. Authored test fixtures remain NOT_EXECUTED.


## Visual feedback while MC02 receives experimental input — source only

The existing `observe_client({slot})` and a previously started bounded
visual observer may capture **identity-verified, read-only** frames during
an explicitly opted-in MC02 action sequence. A separate image-capture
lock prevents concurrent screenshots and preserves the active native
mutation lock. The human MC01 is still excluded in hybrid mode. Each
captured image is checked against exact PID/HWND/birth/session both
before and after Cua/WGC access, and may be discarded if the game
window changes.

Launch and other native mutations remain serialized with screenshots.
The existing emergency neutral MC02 write retains priority over an
ordinary read-only frame when no sequence is active, subject to native
controller identity admission. During a sequence, stop still requests
cancellation and the sequence attempts its own release.
Native IPC contention or a failed image read is reported as unavailable,
not as gameplay evidence. Cua/WGC providers, hooks, packaging defaults
and production input remain unchanged. Tests are authored NOT_EXECUTED.

## Per-slot native input admission (MC02 source candidate only)

The **existing retained Rust session controller** now applies one
`admit_native_input_frame(slot,keys,dx,dy,ttlMs)` rule to the
same-image one-shot CLI, authenticated session IPC, controller
dispatch and MC02 channel writes. No second lifecycle manager or
input provider was introduced. The rule refuses MC01, MC03, MC04
and MC05 even if a forged command bypasses the MCP planner; the only
eligible slot is the separately opted-in MC02 experiment.

The admission also rejects contradictory forward/back and left/right
bits, out-of-range relative mouse values and invalid TTLs. It
distinguishes `Active` from `ReleaseAll` at the native boundary:
release is still authorized only against the original MC02 process
PID/birth/session and channel and does not need human foreground.
Active input still needs MC01's real foreground and exact original
MC02 window/process identity. A native shared-state ACK is not
gameplay input evidence. The external wire schema, private 96-byte
ABI, package admission and normal disabled features remain unchanged.
Authored fixtures are NOT_EXECUTED.


## MC02 background Raw Input thread classification — source only

The existing opt-in **read-only** MC02 API observer distinguishes
`RAW_BUFFER_OGLES_THREAD_POLL` from
`RAW_BUFFER_OTHER_THREAD_POLL` for successful non-null
`GetRawInputBuffer` calls. Previously it required the exact MC02
`OGLES` root HWND to be visible and non-minimized. That suppressed
both markers when a still-running Minecraft window was hidden or
minimized, even if the game thread continued polling. The observer
now classifies using the original process-local PID, top-level
unowned `OGLES` root, and **exactly one matching root** without
requiring visibility. Missing/ambiguous roots still produce no
thread-classification claim; PID/HWND/TID values are not logged.

This changes only that existing read-only observer, not the
virtual-input adapter: **active MC02 movement still requires the
original visible, non-minimized, responsive window and real MC01
foreground**. Neither the observer nor the MC02 input experiment
is activated or packaged by this source change. An observed poll
identifies the Win32 API/thread, not in-game input delivery.
REMOTE_GITHUB source review only; Windows observations NOT_EXECUTED.

## MC02 batch replies avoid unrelated game window title writes — source only

The retained session actor continues returning a truthfully verified
five-profile `SessionSnapshot` after each MC02 native pulse. Previously
every 160-ms heartbeat could also relabel all five Minecraft windows,
even though MC01/MC02 were already checked for the input write. Those
window-title mutations are now suppressed ONLY for MC02 input replies;
the original process/HWND and responsiveness checks are retained.
Normal Inspect, launcher and close paths retain their existing window
labeling and lifecycle behavior. No new input controller, IPC schema,
hook or account access was introduced. This removes unnecessary
Win32 work from the heartbeat path, not proof of improved runtime
latency or Minecraft gameplay. Builds/tests remain NOT_EXECUTED.

## Consistent MC02 virtual polling during batch and release — source only

The existing C++ `currentPacket()` gate used by
`GetAsyncKeyState`, `GetKeyState`, `GetKeyboardState`,
`GetCursorPos` and the three focus-read hooks now samples its
active packet and pinned MC01 HWND inside the same ABI-v4
`cursorRevision` even/odd transaction. It rejects uninitialized,
odd/in-progress and changed revisions, including a revision that
changes while the real (unhooked) Windows foreground is checked.
Before this correction the virtual Win32 polling hooks could read
an old packet and a newly cleared host HWND while native ReleaseAll
was being published. The buffered Raw Input reader already had
a corresponding revision-based guard.

No extra hook, notification loop, process, driver, IPC transport,
ABI version or MC03–MC05 permission is introduced. This is
**source-level coherence only**, not evidence that Minecraft
Education consumes the virtual keyboard, mouse or camera state.
Normal production input remains off; builds and gameplay are
NOT_EXECUTED.

## Opt-in MC02 timed Computer Use action sequence — source only

The existing Codex-facing local stdio MCP has a **separate** experimental
action sequence that links the already-authored semantic timeline to the
existing opt-in MC02 native shared-state writer. It does NOT replace Codex,
install a driver, create a game controller, or enable four/five independent
AI inputs. Normal MCP starts do not advertise these tools.

This requires an explicitly reviewed Windows candidate built with the
matching optional Rust `mc02-native-input` feature and the existing
MC02-only compatibility DLL input hook, plus **both** the MCP
`--mc02-input-experimental on` option and the environment
`LAZY_TESTER_MC02_VIRTUAL_INPUT=I_ACCEPT_EXPERIMENTAL_MC02_INPUT`.
Do not enable those experimental binaries or modify installed Minecraft
without separate operator authorization. A source commit alone cannot
make them executable.

- `run_experimental_mc02_actions` accepts at most 12 semantic actions
  targeting **MC02 only**, a 3.5-second maximum timeline and existing
  nonoverlapping channel rules. It supports move, strafe, jump, sneak,
  attack, use, and `look_relative` (individual deltas at most ±127).
- It compiles through the existing `input-timeline.mjs`, coalesces
  same-timestamp transitions into complete held-button states, and
  prepares bounded pulse times every 160 ms with TTL 350 ms. Camera
  deltas are published only once. A terminal camera-only action holds
  that one-shot native packet through its authored 50–250 ms duration,
  then schedules an ordinary neutral MC02 release. Without that dwell,
  the previous immediate ReleaseAll cleared the unconsumed cursor
  backlog before the next possible game poll. The entire camera action
  and its release must fit within the original 3.5-second horizon;
  this creates no extra look movement and does not prove Minecraft
  actually polls or applies the delta. A simultaneous key action may
  overlap a camera action **only while its eventual native neutral
  release does not precede the camera's authored exposure end**.
  Since the existing zero-input `ReleaseAll` clears pending camera
  totals, the batch compiler rejects an early key-up/neutral pulse
  that would truncate any overlapping `look_relative`. It does not
  extend key holds, replay camera deltas, or claim the game consumed
  them. **One** opt-in same-image
  `mc02-input-batch` CLI subprocess schedules the entire sequence;
  each tick still enters the **same** authenticated retained controller.
  This avoids process startup for every heartbeat. An initial wait is
  preserved: the first pulse may have a positive time offset, and no
  artificial 0-ms command is sent before the requested action begins.
  No second controller, new pipe, driver, game hook, or server is introduced.
- The native bounded batch refuses a stale initial request, nonmonotonic
  timestamps, conflicting controls, out-of-range camera deltas, late
  pulses (>150 ms) and all non-MC02 input. The CLI checks the SAME
  150-ms monotonic deadline both **before dispatch** and **after IPC
  receipt**: even a slow final heartbeat acknowledgement cannot be
  reported as an on-time sequence. After a late receipt, the batch
  stops and attempts its existing exact-owner neutral release, as
  it does on normal completion and other handled errors. Transport
  timing does not prove when Minecraft consumed game input. After the process
  exits or is cancelled, MCP independently attempts another native
  neutral write, with TTL expiry as the final fail-safe when killed.
  An exact-owner, fully neutral MC02 request may also pass an
  *uncertain previous IPC* flag in the same CLI process. A still-pending
  native worker is never bypassed; all new movements and unrelated
  mutations remain denied under uncertainty. The original controller
  still checks MC02 PID/birth/session and the opted-in channel.
  The existing MC02 native channel now also retains the greatest
  monotonic `GetTickCount64` issue timestamp of any neutral release.
  This closes a cross-process cancellation race: a delayed older
  active pulse arriving *after* ReleaseAll is rejected even if it is
  still under the 500-ms normal request-age limit. The channel also
  remembers the latest **active** input issue time. An older active
  pulse arriving after a newer one cannot overwrite its keyboard or
  camera state; a delayed old ReleaseAll cannot erase a newer accepted
  active pulse. A same-millisecond tie gives neutral ReleaseAll
  precedence over active state. Both watermarks are controller-local,
  not another provider or change to the 96-byte shared-memory ABI.
  A newly issued command after release still passes the original
  per-slot admission and focus/ownership gates.
  For one-shot `mc02-input` calls, the CLI now preserves the original
  MCP `requestedAtMs` request order by translating its validated
  <=500-ms wall-clock age into same-host `GetTickCount64` time.
  Previously an older neutral command delayed during Windows process
  startup could obtain a newer issue timestamp, circumventing the
  controller's two existing cross-process ordering watermarks.
  Unrepresentable uptime and invalid/future/stale wall time refuse
  the request. This local conversion is bounded and conservative,
  not evidence of input timing or Minecraft gameplay; independent
  160-ms batch heartbeats still use their actual dispatch uptime.
  If native release cannot be delivered, TTL remains the fail-safe.
  The output remains **transport acknowledgement only**; actual
  Minecraft gameplay and keyboard release remain NOT_EXECUTED.
- `stop_experimental_mc02_actions` requests cancellation while a sequence
  is running; that immediate response is NOT a release acknowledgement.
  When no sequence is active, it issues one neutral MC02 native write
  (also useful after a stand-alone pulse) through the same authenticated
  controller. Native cleanup requires MC02's pinned original PID/birth/
  session and an opted-in channel, **not MC01 foreground**, because loss
  of human focus must not prevent release. The underlying shared state is
  cleared (active packet, cursor backlog, press counts and human-focus hint)
  under the existing ABI-v4 revision. The last pinned MC02 HWND is retained
  inside its private channel until process close, so an existing
  `GetRawInputBuffer` game-thread consumer can still receive pending
  KEY_BREAK and mouse-button UP records after explicit cancellation.
  An inactive packet cannot spoof the game focus or authorize movement.
  This is a transport state acknowledgement, never proof that Minecraft
  consumed a key-up. The existing opt-in C++ buffered reader now
  validates the ABI-v4 even/odd revision for **neutral/expired** packets
  as well as active packets. After draining the physical queue it
  rechecks both that revision and the MC02-owned game window before
  copying any synthetic release or input records. Rejected or partial
  reads do not advance the held-key/mouse state or cursor counters, so
  subsequent eligible polls can retry the remaining release records.
  The buffered candidate now uses a two-phase shared button
  planner: **release all previously held keyboard/mouse controls
  first**, then deliver fresh press transitions and finally the
  one-shot relative-camera record. With small buffers, a new WASD
  press cannot precede an already pending attack/use release.
  Existing press counters reconstruct any undelivered suffix; each
  button's release-before-press and short click down/up semantics
  remain intact, and the cursor advances only if its movement record
  was actually copied.
  The opt-in Raw Input reader may also identify a hidden/minimized
  MC02-owned OGLES window for pending key-up/mouse-up records after
  packet expiry, focus loss or explicit ReleaseAll, matching the
  existing native controller's release-only admission. The active
  input and foreground/active/focus hooks still require a visible,
  non-minimized window; no window is activated or switched.
  Native TTL remains a
  fail-safe if the release cannot be delivered. The normal single-pulse
  tool and unrelated normal tools remain unchanged.

An MC02 native write receipt establishes **transport acknowledgement
only**. It does not prove Minecraft Education accepted WASD, rotated
camera, attacked, stopped, joined a world, or preserved MC01 input.
Prioritize one live MC01-human/MC02-AI observation at a separately
authorized Windows acceptance stage before enabling other clients.
Authored source fixtures are registered with the existing optional
`test:localtest` script and have NOT been executed.

## Phase 1: Codex-targeted Windows Computer Use (source only)

New MCP tools `get_computer_use_target({slot})` and
`revalidate_computer_use_target({slot,pid,windowId,createdFileTime})`
resolve one of the existing five native Minecraft windows through the same
authenticated session controller. The native Rust adapter verifies exact
PID, process birth FILETIME, Windows session and root HWND using the existing
window owner, and refuses unavailable, minimized or nonresponsive windows.

In `human-plus-four-ai`, MC01 is **never** offered as an AI action target;
in `five-ai`, MC01 can be targeted. The response contains a **Cua Driver**
window selector `{kind:"window",pid,window_id}`. No external software is
automatically installed or started. Configure Cua Driver as a *separate*
Codex/Claude MCP server only after explicit local-setup approval; this
repository currently adds only reliable native target discovery.

Proposed agent-side read-only workflow after installation approval:

1. Resolve `get_computer_use_target({slot:2})` in Lazy Tester MCP.
2. Call Cua Driver `get_window_state({pid,window_id})` using the returned
   exact selector. Inspect the screenshot and accessibility state.
3. Revalidate the same `pid/windowId/createdFileTime` after capture.
4. If the target changed, discard the image or action; do not silently
   follow a replacement process. In hybrid mode never escalate Cua to
   foreground or use global SendInput, even if the background action refuses.

**No screenshot is captured by Lazy Tester in this phase**. Cua Driver is
an optional external window-capture provider, not a Minecraft game-input
backend. The reported verification applies only at the instant of native
inspection; it is not an authorization token for arbitrary external tools.
Background screenshot availability, all game input, P2P host/join, and
multiple simultaneous AI players remain NOT_EXECUTED / UNVERIFIED.
The existing MCP controller, UI, world storage and disabled CI are unchanged.

Official API basis:
- https://cua.ai/docs/cua-driver/concepts/how-cua-driver-works
- https://cua.ai/docs/cua-driver/reference/mcp-tools/apps-windows-and-screen
- https://learn.microsoft.com/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createforwindow

## Phase 2: read-only visual observation via Cua Driver (source only)

An optional **`observe_client({slot})`** MCP tool produces one
`image/png` content block and compact metadata after selecting the exact
AI-owned Minecraft window via the retained native controller. This uses
the already documented Cua Driver read-only CLI, avoiding a new WinRT/D3D
capture engine and avoiding any input/control into Minecraft.

To enable this *after separate operator authorization for installation and
runtime acceptance*, add `--cua-exe` with the absolute path to an ordinary
`cua-driver.exe`, for example `C:\\Tools\\cua-driver.exe` (example
path only, not a claim about the installed location). A separately installed
Cua Driver daemon must already be running; this bridge never starts it.

`observe_client` performs:
1. Resolve the current exact PID/HWND/birth/session for the AI-owned slot.
2. Call **only** `cua-driver call get_window_state` with explicit
   `pid` and `window_id`, screenshot enabled and accessibility walk
   disabled; use Cua's documented `--screenshot-out-file` option.
3. Read one PNG capped at 1280 px long edge (strict absolute limit 1536 px
   and 6 MiB); erase the temporary file before returning the MCP image.
4. Resolve the exact target again. Reject the frame if PID/HWND/birth/session
   changed. Missing Cua, minimized/stale windows, missing frames and unexpected
   image data fail closed; no desktop-wide fallback or foreground switch.

MC01 remains protected in `human-plus-four-ai`; `five-ai` may observe
MC01. One capture is **not proof of a live frame or successful gameplay**;
stale, black or unobtainable game frames must be reported as uncertainty.
No keyboard, mouse, cursor overlay, computer-use action, world mutation, or
automated agent scheduler is introduced. Cua's read-only CLI must be
reviewed and accepted on the eventual Windows installation, and capturing
a game-specific GPU window is still `NOT_EXECUTED`.

The PNG is stored only temporarily while servicing the request, not in a
world, credential/profile folder or Git repository. An abrupt process
termination could leave a temporary file for the operator to clean later.
Only code is committed; no user screenshots are collected now.

Cua CLI reference: https://cua.ai/docs/cua-driver/reference/cli/serve-and-configure

## Phase 19: native five-profile status projection at the MCP boundary

The canonical Rust `engine/agent_cli.rs` already constructs a
minimal inspect/launch payload with only `slot`, `state`,
`hasWindow`, `responding` and `needsAttention` per profile.
No process path, account, world folder or private issue text is
included in the actual Rust CLI output. **This is defense in depth,
not a demonstrated leak from the current Rust product.**

The existing Node `validateNativeReport` previously checked those
five fields but returned the **original native JSON**. The direct
`inspect_profiles` and `launch_profile` MCP actions bypassed
that validation altogether when the native implementation returned
a report. That would expose unneeded additional fields if a future
native response or injected provider included them.

The Node validator now reconstructs one exact five-profile view and
forces the truthful `NATIVE_SESSION_ONLY_NOT_GAMEPLAY_OR_ACCOUNT_SIGNIN`
evidence marker. The inspect and launch tool handlers reuse the same
validator; `get_test_context` and `plan_peer_session` were already
routed through it, so their profiles are now projected as well.
Malformed core state/slot/operation still fails closed. Explicit
profile launch is still an existing, permission-gated native action,
not an implicit game launch.

Phase 18's text-size guard remains present. Its existing oversized
native-stub fixture was adjusted to expect a small sanitized report,
and a direct pure serialization check retains coverage of the 128
KiB limit. New cases in the same test file cover private native
fields in both direct/MCP inspect/launch, context output, and invalid
core profile state. These tests are AUTHORED_NOT_EXECUTED.

No Rust controller, profile/world storage, account sign-in, native
window presentation, network protocol, additional MCP tool, Windows
installer, input backend or CI definition was modified. Real
Windows/Minecraft acceptance remains NOT_EXECUTED and AI gameplay
input remains BLOCKED.

## Phase 18: bounded MCP responses and private evidence error handling

The existing `read_game_evidence` MCP tool previously returned the
raw parsed Skills Probe `data` object. Because the capture format
allows additional unrecognized JSON keys, this could send internal
debugger fields, account paths or other unneeded content to a model.
The tool keeps its name, slot/status/events response structure,
but now reuses the existing `summarizeEvidenceReports` allowlist
projection. Only bounded event fields are exposed; arbitrary nested
log JSON is not. This is a deliberate narrowing of the public event
shape. `summarize_test_evidence` remains available for cross-slot
reports, and `readGameEvidence` remains internal to those reports.
Short sanitized metadata strings can still contain user-supplied
values: they are UNTRUSTED, not proof of identity or secrets filtering.

MCP JSON-RPC tool errors previously returned `error.message`
directly. Provider or filesystem errors can include local paths,
user data or debugger text. Now only fixed, known application
error messages and constrained native refusal codes are returned;
unexpected failures use `Tool unavailable; details withheld`.
Direct source `callTool` exceptions remain unchanged for regression
assertions; this protection applies to model-visible MCP responses.

The existing JSON-RPC response boundary now caps tool text at
128 KiB and image base64 at 8 MiB (compatible with the 6 MiB Cua
one-shot PNG ceiling). An oversize result is an explicit error with
no partial payload. The source tests (AUTHORED_NOT_EXECUTED) cover
extra private log fields, leaked provider paths, known MC01
protection errors, and overlarge text/image outputs. No new MCP tool,
game lifecycle, persistent state, credential store, Windows input
provider, CI or daemon was added.

## Phase 17: reject incidental markers and untyped results

The previous Skills Probe reader accepted the substring
`[ANALYTICS]` or `[GAME_RESULTS]` anywhere within a PrintEvent
message. An ordinary chat/debug line referring to those markers could
therefore be parsed as a game event. It also accepted arbitrary JSON
objects after `[GAME_RESULTS]`, and the evidence summary counted every
caller-supplied `game_results` object as a `RESULT`.

The existing `parseCapturedLine` now requires a valid-looking,
parseable capture timestamp, one **leading** Skills Probe marker
(after decoding the established JSON-string-escaped PrintEvent format)
and one bounded JSON object. Analytics require a non-empty `type`.
The currently recognized game-result shape is
`{"schema":1,"record_type":"gameplay_result",...}`; untyped,
unknown-version or other result objects are ignored. The summary
performs the same minimal result-type check because its internal
callers need not use the file parser.

`resultCount` remains a count of recognized **untrusted records**,
NOT success, winning, correct host/join, independent AI movement, or
verified player/account/world identity. Historical analytics with a
non-empty `type` and no schema are still read as unverified telemetry.
This compatibility choice must be reviewed against real Skills Probe
output before claiming producer-schema acceptance.

Existing regression files cover embedded markers, untyped/malformed
results, schema mismatch, trailing JSON garbage, invalid timestamps,
and unchanged valid analytics/canonical result examples. All tests are
AUTHORED_NOT_EXECUTED. No new parser service, persistent state, game
hook, input backend, CI or native lifecycle component was introduced.

## Phase 16: stable bounded Skills Probe log snapshots

The `readGameEvidence` reader previously took its tail offset from
`lstat(path)`, then separately `open(path)` without checking they
referred to the same file. An MC02 log replaced/rotated during this
interval could cause an offset from the old file to be applied to the
new file, producing incomplete or misattributed records. A log being
appended/re-written during an open read had no before/after snapshot
gate either.

The existing reader now pins an ordinary file handle and compares its
nonzero device/inode identity, size, nanosecond modification/change
timestamps, and pathname identity before and after a bounded positional
read. A changed, disappearing, or unverifiable source returns
`CAPTURE_FILE_UNSTABLE` with **zero events**; the summary preserves
that status as a missing/unaccepted source. It never guesses that the
previous or replacement log belongs to the same Minecraft player.
Filesystems without a verifiable file ID fail closed. The reader
retries short positional reads within its existing 256 KiB limit, and
still drops trailing incomplete lines; the returned metadata flags
`incompleteTrailingLineIgnored`.

This is a best-effort atomic *reader-side* snapshot check, not
proof that the external debugger wrote a transactionally consistent
record or that account/world attribution is correct. Deterministic
regression fixtures cover rotation before open, append during read,
short reads, partial tails and unstable-source summarization. They
are AUTHORED_NOT_EXECUTED under current operator policy; there is
no new watcher, permanent log index, file store or Minecraft hook.

## Phase 15: bounded visual-worker startup and evidence isolation

The optional WGC source previously checked active workers only BEFORE its
asynchronous executable-file inspection. Concurrent starts for a slot, Stop
during this wait, or MCP connection shutdown could allow an untracked
late worker. `gpu-frame-source.mjs` now reserves each slot before the
await. Stop/shutdown revoke pending reservations; cancelled starts refuse
to spawn a process. Failure to supervise a spawned worker triggers a
kill attempt; failures while attaching worker handlers/timers also drop
the worker. These actions affect **capture children only**, not Minecraft.

MCP `stop_visual_observation` now cancels an overlapping, unfinished
multi-slot WGC startup batch rather than reporting an unqualified Stop.
The response explicitly reports
`STARTUP_CANCELLATION_REQUESTED` with an unconfirmed outcome until
the original Start call resolves. The existing observer immediately
terminates on a confirmed GPU/native window identity-change error,
clears the old frame, and runs its existing worker cleanup callback.

Skills Probe record IDs are not globally unique across all Minecraft
sessions. `evidence-summary.mjs` now scopes deduplication to the
reported `session_id` as well as slot and source; events without a
session ID are kept distinct rather than silently merged across
unknown sessions. File-to-player/world attribution remains
`OPERATOR_NAMED_UNVERIFIED`.

All changes reuse existing GPU, MCP, visual sampler and evidence owners;
only existing regression files were extended. Tests are AUTHORED
but NOT_EXECUTED. Windows capture, real game session identity, release
of a hung native process, account login and multi-client gameplay remain
NOT_VERIFIED. No extra service, installer, input backend or CI added.

## Phase 14: pin GPU frames to their original native client (source only)

The optional WGC worker is created for an exact, native-verified
Minecraft PID, HWND, process birth FILETIME and Windows session. The
previous `gpuSource.latest(slot)` checked packet age, while MCP checked
the *current* native target before/after the call, but did not verify the
worker was originally started for that exact target. A restarted MC02
could therefore be incorrectly attributed a stale frame captured by
an older worker when the old worker remained active.

`gpuSource.latest(slot,expectedNativeTarget)` now requires a fresh exact
binding and compares all four identity fields against the worker's
original pinned target. Any mismatch immediately discards the cached
PNG and stops that capture worker. The MCP passes its already-verified
native target into every WGC read, then still revalidates the target
after the frame read. Future-dated frame timestamps are refused until
the capture time has actually arrived.

The existing tests now cover PID, HWND, birth and Windows session
replacement, required binding, and future frames; MCP tests assert the
correct native identity is passed to the GPU cache. They are
**AUTHORED_NOT_EXECUTED**. No new tool, continuously running daemon,
input backend, driver, game control or profile lifecycle component
was added. A source-level identity guard is NOT proof that WGC can
capture Minecraft Education gameplay on Windows.

## Phase 13: Windows Agent Workspace + Codex interoperability verdict

**Outcome: not a production input provider, no new implementation.**

Microsoft's Windows On-Device Agent Registry (ODR) can run an
MSIX/package-identified binary MCP server under a separate agent user and
Windows session. The contained MCP server may launch executables **within
that same agent session**, but cannot directly inspect or control the
human's window/process/session. This is a useful official *process
boundary*, not a documented game-oriented keyboard/mouse delivery API.

There are three distinct integration paths; do not conflate them:

| Integration | Actual published capability | Lazy Tester decision |
| --- | --- | --- |
| Current Codex + local Node stdio MCP | Invokes existing same-session controller, plans/observes clients; no native game input | **Keep**, no new tool |
| Windows ODR + contained MCP (.exe/MSIX identity) | ODR registers packaged tools and launches them under agent-account session; may run in-session executables | **Not wired to Codex**; generic game input/capture and five-client session lifecycle unverified |
| Windows 365 for Agents computer-use MCP | Desktop click/type/screenshot/session lifecycle against a remote Cloud PC | **Not eligible** for local/no-cloud design |

ODR `odr.exe` registration is **not** evidence of a per-process keyboard
or relative-mouse transport, and package identity alone does not turn
Minecraft Education into a remotely controllable game. Likewise,
Windows support for a separate agent account does not prove that four
independent game sessions can remain concurrently playable.

The following independently documented or observed conditions would be
necessary before reopening this route: a supported third-party local
agent-session lifecycle with a verified Codex host path; keyboard,
relative mouse and visual capture within one agent session; the
Minecraft Education client rendering and accepting gameplay in that
session; preserved human MC01 input and independent account/world
ownership; then four AI sessions and P2P gameplay. These are currently
**UNVERIFIED/NOT_EXECUTED**. Do not build a new MSIX, ODR adapter,
session broker, input driver, or second Minecraft controller solely
to satisfy a hypothetical workflow. Do not turn on "Reduce protections
for agent connectors".

Windows 365 desktop computer-use documentation is a **different
cloud product**, not a substitute for missing on-device ODR input APIs.
The source-only decision closes Agent Workspace as an immediate coding
task; it does not reject future supported APIs.

Verified official references:
- https://learn.microsoft.com/en-us/windows/ai/mcp/servers/mcp-containment
- https://learn.microsoft.com/en-us/windows/ai/mcp/servers/mcp-server-overview
- https://learn.microsoft.com/en-us/windows/ai/mcp/odr-tool
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features
- https://developers.openai.com/codex/mcp/
- https://learn.microsoft.com/en-us/microsoft-copilot-studio/mcp-windows-365-agents

**Proof ceiling:** REMOTE_GITHUB source/API review. No tests, builds,
ODR actions, Windows installation, Codex sessions, Minecraft gameplay,
or input were executed. Production dispatch remains disabled.

## Phase 12: native Windows multiseat feasibility decision (source only)

**Decision: No multiseat runtime implementation is admitted yet.**
We examined native controller ownership, official Windows/Minecraft
platform support and a community RDP-based multiseat implementation.
No available path currently satisfies the product's simultaneous
MC01 human + MC02–MC05 independently controlled AI acceptance.

| Windows design | Source-supported fact | Decision for Lazy Tester |
| --- | --- | --- |
| Extra Win32 desktops in the same session | Only one input desktop is active at a time | Does not provide four independently focused games |
| Windows Server RDS | Multiple interactive RDS sessions, but Minecraft Education **client** explicitly does not support Windows Server | Exclude |
| Windows Enterprise multi-session | Microsoft's licensing does not allow production deployment outside Azure Virtual Desktop | Exclude from local no-cloud/no-VM design |
| MultiSeat/RDPWrap on Windows 10/11 | Uses RDP loopback, RDPWrap, streaming components, and privileged system modifications; its optional K/M InputHook is explicitly documented as a no-op | Research reference only, no integration/installation |
| Windows Agent Workspace / ODR | Preview agent identity runs in separate Windows session; contained MCP must be a packaged .exe with MSIX identity and cannot directly control a human session | Most relevant first-party research candidate, NOT a validated game controller |

A session-local input desktop is a **plausible** way to separate AI
controls from the human console session, but Microsoft Agent Workspace
does not establish either four independent agent sessions or acceptance
of Minecraft Education game input/graphics. Microsoft's contained MCP
server model also differs materially from Lazy Tester's unpackaged
Node stdio MCP: no automatic packaging, ODR discovery from Codex, game
startup or cross-session bridge exists. Do not use the existing
`--lazy-tester-agent-cli target` process lookup to cross sessions.

**Required proof order before any implementation:** (1) documented,
license-compliant and approved way to run ONE isolated Windows agent
interactive session under the user's target Windows edition; (2) one
independently licensed Minecraft Education client launches/signs in,
renders, and accepts keyboard/relative mouse entirely inside that session;
(3) human MC01 continues to receive uninterrupted input; (4) world
host/join, game-state evidence and native identity are attributable to
the two respective accounts; (5) prove four independently addressable
AI sessions and resources before promising a four-AI or five-AI mode.
Any failure remains `BLOCKED`, not a "source test PASS".

Neither package creation, new service, RDPWrap, driver, agent workspace
activation, input injection, Minecraft nor source tests were run.
MCP still exposes read-only route planning but no gameplay actuator.

Official/maintainer references:
- https://learn.microsoft.com/en-us/windows/win32/winstation/desktops
- https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide
- https://learn.microsoft.com/en-us/azure/virtual-desktop/windows-multisession-faq
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features
- https://learn.microsoft.com/en-us/windows/ai/mcp/servers/mcp-containment
- https://github.com/vibesoftwarecoder/MultiSeat/blob/master/README.md

## Phase 10: Windows per-client game input feasibility (source-only)

Read-only native PID/HWND identification is NOT a channel for delivering
independent Minecraft movement to that window. The existing session
controller and Skills Probe evidence owners remain unchanged.

| Candidate | Observed limitation | Decision |
| --- | --- | --- |
| `SendInput` / Windows `InputInjector` | Shared system input, no external exclusive game PID target | Not an independent input backend |
| Raw Input `RIDEV_INPUTSINK` | Receiving game must register itself for background input | Not an external sender |
| GameInput `SetFocusPolicy` | Focus policy belongs to the game API consumer | Not an external process router |
| ProtoInput | Can reroute inputs using hooks inside game processes | Not authorized in no-injection design |
| Windows Agent Workspace | Preview separate Windows accounts/sessions | Research only: native controller has no cross-session route |
| ViGEmBus / XInput | Retired driver, globally visible device and four standard indices | Historical source retired from Local in Phase 11 |

**Phase 10 source correction:** `assess_input_route` rejects any
cross-Windows-session target combination in BOTH hybrid and five-AI
modes. Windows Agent Workspace cannot be used as a drop-in input provider
for current same-session Lazy Tester. It would require a separate
architectural decision and target-session ownership proof. Neither
support for four independent Minecraft game sessions nor GPU rendering,
Education sign-in or P2P connectivity has been demonstrated.

Minecraft Education documents keyboard/mouse controls and a limited
controller settings screen, while a later Education support response
states Windows computer gameplay is keyboard/mouse only. Therefore
controller acceptance remains **UNVERIFIED**; keyboard and relative
mouse input are the required capabilities.

**Future acceptance (not performed):** MC01 remains human-controlled
and uninterrupted; independent MC02 moves, looks and attacks while
unfocused, with game-frame/Skills Probe evidence attributed to MC02.
No external driver, client injection, local tests or gameplay ran.

Sources:
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- https://learn.microsoft.com/en-us/windows/uwp/ui-input/input-injection
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice
- https://learn.microsoft.com/en-us/xbox/gdk/docs/reference/input/gameinput/interfaces/igameinput/methods/igameinput_setfocuspolicy
- https://www.splitscreen.me/docs/proto/
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features
- https://edusupport.minecraft.net/hc/en-us/articles/360047116832-Minecraft-keyboard-and-mouse-controls
- https://edusupport.minecraft.net/hc/en-us/community/posts/53960255250324-cant-connect-ps5-controller-to-play

## Phase 8–9: retired virtual-gamepad experiment

The optional ViGEm/XInput worker and offline mapper were **removed from
the active `Local` source during Phase 11**. They were never production
input providers, were never built or installed, and were never connected
to the MCP. Their exact source remains recoverable at immutable commit
[`fc4d87d9defe`](https://github.com/OpenITStudios/M-Minecraft-Emulator/tree/fc4d87d9defe01eafd87a0abc6904f2f9d7d5807/Development/tooling/agent).

Retirement reason: the candidate emitted a globally visible virtual
gamepad rather than keyboard/relative-mouse input exclusively bound
to MC02–MC05. Standard XInput has four device indices; ViGEmBus is
retired. Minecraft Education's 28 September 2026 support response says
computer play currently supports keyboard and mouse. Older Education
documentation mentioning controller settings is contradictory, so an
XInput claim cannot be promoted without actual game acceptance.
There is no installed component to remove from user machines.

Removing this lab also removes its experimental test from the existing
`test:localtest` command and its irrelevant diagnostic field from
`assess_input_route`. The tested semantic gameplay planner, read-only
PID/HWND route, safety gate, optional MCP observation, native controller,
profile/account/world data and all live acceptance requirements remain
unchanged.

## Phase 7: bounded independent input transaction runner (source only)

The source-only `input-delivery.mjs` is a narrow **per-client native input
provider port**, not an installed keyboard/mouse driver and not a second
Minecraft session controller. It is **NOT registered** as an executable MCP
tool. Only read-only `assess_input_route` and existing planning/observation
tools are exposed to Codex.

The source harness accepts internal callbacks that a future validated native
provider must implement: `verify(slot)`, `admit(target)`, `send(target,event)`,
`releaseAll(target)`, and `protectHuman()`. It cannot be enabled by passing
an MCP tool parameter. The production MCP always reports
`provider:NONE`, `execution:DISABLED`, and
`BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`.

Safety properties implemented in **source**, not demonstrated on Windows:

- One bounded transaction; no overlapping runs, at most 64 semantic events
  within 35 seconds. A native route older than 1.5 seconds is refused.
- Recheck the exact PID/HWND/birth/session identity before each timestamp
  group. In hybrid mode a separate human-desktop protection callback is
  mandatory. The MC01 slot is never an AI input target.
- Parallel **between different MC slots**, ordered **within one slot**:
  release a previous button before pressing its replacement; relative
  camera deltas are separate events.
- Cancellation and transport errors enter a `finally` block that calls
  `releaseAll` for every target. Incomplete release is a hard error.
  The provider must additionally implement its own crash watchdog:
  JavaScript alone cannot guarantee hardware release if the process hangs.
- Even acknowledged provider callbacks are reported as **transport only**;
  they never establish gameplay movement, camera control or human-input
  safety in Minecraft.

**Critical remaining blocker:** An independent, non-foreground native input
transport that works with Minecraft Education has not been found or
accepted. Windows `SendInput` is a shared system stream; Cua Driver's
Win32 background action does not guarantee game controls; ProtoInput's
multigame routing uses game-process hooks. Microsoft's isolated agent
workspaces exist but have no demonstrated five-Minecraft integration.

No DLL injection, driver installation, desktop-wide `SendInput`,
foreground-swap, MakeCode, gameplay or executable MCP input action was
added in this phase. The test file uses internal callback fixtures to
exercise sequencing and safety only; it is **NOT_EXECUTED**. A passing
fixture in the future would not count as Windows or Minecraft acceptance.

Relevant research:
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- https://www.splitscreen.me/docs/proto/
- https://cua.ai/docs/cua-driver/concepts/how-cua-driver-works
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features

## Phase 6: native-bound independent input admission (source only)

**New read-only tool:** `assess_input_route({actions})` connects the
existing concurrent input timeline to freshly observed native Minecraft
window/process identities. In `human-plus-four-ai` it also inspects
MC01's window **only as a protected human reference**, never as an
AI input destination. In `five-ai`, MC01 may be an AI target.

The MCP reuses its current, retained authenticated controller and
`--lazy-tester-agent-cli target <slot>`; each target must pass the
same PID, process creation FILETIME, session, root HWND, visible and
responsive checks as visual observation. A route is rejected if two
slots resolve to the same PID/HWND, any target is unavailable, or the
overall native observation spans more than five seconds. This is an
ephemeral diagnostic mapping, **not an input authorization token**.

Every response explicitly reports:
- `NOT_DISPATCHED` and
  `BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`.
- The AI-only PID/HWND/window identity map, plus an observational
  (not guaranteed) human MC01 foreground indicator.
- The independent capabilities that a future native provider must
  demonstrate: genuine per-client keyboard, relative mouse, concurrent
  clients, release-all, preserved human input, and actual Minecraft
  game-state evidence.

No actual keyboard, mouse or game action is delivered by this tool.
Windows `SendInput` targets the shared system input stream, not a
specified Minecraft process. `RIDEV_INPUTSINK` does not allow an
external MCP to force another application to accept raw input; the
receiving application must register. Cua's background UI input does
not establish raw game control, while ProtoInput requires in-process
hooks whose use with Minecraft Education has not been approved.

Relevant sources:
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice
- https://cua.ai/docs/cua-driver/concepts/how-cua-driver-works
- https://www.splitscreen.me/docs/proto/

Source regression fixtures were authored for hybrid/5-AI slots,
aliasing/refusal and truthful NO-INPUT reporting. **No tests, local
builds, game sessions, CI, or external input processes were run.**

## Phase 5: concurrent independent-input contract (source only)

The previous planner refused **any** simultaneous controls for one Minecraft,
which would prevent normal AI gameplay. `plan_player_actions` now allows
overlapping *independent channels*: forward/back movement, lateral movement,
camera, jump, sneak, attack, and item use. Conflicting directions on the
same axis and exclusive waits still cannot overlap. Different MC slots may
use different controls concurrently.

A new `look_relative` action accepts integer `deltaX/deltaY` within
±512 and a 50–250 ms scheduling window. It is **not calibrated** to Minecraft
mouse sensitivity or an actual input device. Request
`plan_player_actions({actions,includeTimeline:true})` to compile ordered
PRESS/RELEASE/POINTER_DELTA events grouped by timestamp. The default
returns only the existing concise plan. `input-timeline.mjs` can also
prepare a per-slot emergency-release intent.

**Nothing is executed.** All outputs remain `NOT_DISPATCHED`, with
`BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`. The future Windows
provider must independently route actual controls to each game, map game
keybinds, revalidate PID/birth/session/HWND, avoid the human foreground,
and implement physical per-client release-all on errors/disconnect.
A sent-input ACK would not prove a Minecraft player moved.

ProtoInput and Nucleus use in-process game hooks; no Minecraft Education
compatibility proof exists. No hooks, DLL injection, device driver, global
SendInput, MakeCode, UI, second game controller or executable MCP input
tool was added. Tests are authored but NOT_EXECUTED by operator instruction.

## Phase 4: opt-in persistent GPU frame stream (source only)

An isolated Rust worker source using Windows Graphics Capture (`wgc 3.1.0`)
and a bounded Node frame reader now exist under:
- [Native WGC worker](./native-gpu-capture/README.md)
- [GPU frame protocol and reader](./GPU_CAPTURE.md)

Existing `start_visual_observation`, `read_visual_observation`,
`stop_visual_observation` and `observe_client` tools work with an
**optional** persistent WGC provider once its separately reviewed binary
exists. Opt-in configuration: `--gpu-capture-exe` pointing to an absolute
`lazy-tester-gpu-capture.exe`. If specified, this provider takes precedence
over Cua and requires explicit start before a frame is available. The default
remains Cua snapshots when configured, with unchanged legacy behavior.

The MCP verifies each slot via the retained native controller before
starting one bounded worker per selected Minecraft and revalidates PID,
process birth, session and HWND before/after reading each cached GPU frame.
Each worker independently re-verifies the Windows process identity before
emitting. One in-memory frame per window, rate capped at five source
frames/second by MCP, local sampler reads approximately twice per second;
no images sent to Codex unless it explicitly asks for a changed sequence.

**This is neither full-motion video to Codex nor verified Minecraft GPU
performance.** The WGC crate has not been dependency-resolved/built and
the standalone worker is not packaged or installed. Native graphics consent,
Windows capture border, frame availability when Minecraft is backgrounded,
memory/FPS cost and multi-client behavior remain UNKNOWN. No fallback to
a desktop-wide screenshot, global SendInput, native injection, or gameplay
script exists. The worker terminates on stop, expiry, changed target, repeated observer
errors and MCP shutdown; it is not kept alive after an observer finishes.

All local tests, compiler runs, packaging and game execution remain prohibited
until separate operator approval.

## Phase 3: bounded continuous local visual observation (source only)

The existing MCP process can now run a **time-limited rolling frame sampler**
without Codex requesting a fresh screenshot for every observation.
`start_visual_observation` starts in-process sampling for 1–5 AI-owned
windows (MC01 prohibited in hybrid mode). `read_visual_observation`
returns the latest *changed* cached PNG only when Codex requests it, and
`stop_visual_observation` immediately clears the in-memory frame cache.

- Default: one read-only Cua window capture attempt every 1250 ms per slot,
  with slower sampling while the PNG bytes remain identical. Allowed interval
  750–5000 ms, session duration 5–90 seconds. No overlapping captures;
  native controller operations share the existing gate.
- One cached frame at most per client, each at most 4 MiB of encoded data.
  Maximum five independently observed windows. Single frame copy is never
  interpreted as verified gameplay progress, semantic motion, or a real
  30/60-FPS video stream. PNG byte hash change only signals changed **bytes**,
  not a detected enemy, movement or actual pixels.
- Cua preview capture requests `display_only:true`, preserving any
  interactive Cua action snapshot (a visual frame is not an action target).
  A cached frame older than 3500 ms is never returned as a new image.
- Old or replaced PID/birth/session/HWND terminates that slot's sampler.
  Three consecutive capture failures also stop it. Busy native operations
  skip sampling instead of interrupting game lifecycle.
- Timers stop on explicit stop, expiry, and MCP stdio connection closure.
  Source images are ephemeral in memory; not written to persistent
  account/world files or automatically forwarded to an AI model.
- `read_visual_observation({slot,afterSequence,includeImage})` returns
  status only if the newest sequence is unchanged, avoiding repeated model
  image ingestion. The model can combine changes with existing analytics
  summaries through `get_test_context` when needed.
- No background-control keyboard, mouse, camera-rotation, inference model,
  camera stream provider, second daemon, auto-join, or UI is created.
  Cua CLI remains a conservative **bounded frame sampler**, not continuous
  GPU video capture. For genuinely low-latency PvP, a later single persistent
  Windows Graphics Capture frame source would be needed, after proof.

**No local tests or Minecraft live verification are authorized yet.**
Both Cua Windows performance and real multi-window frame freshness remain
UNKNOWN. This code is an opt-in read-only local observation tool.

## One-call AI test context

The first MCP choice for Codex/Claude analysis should be
`get_test_context` (optional 1–5 slots). It obtains exactly one current
native `Inspect` through the existing authenticated session owner, then
combines the conservative host/join plan and compact Skills Probe evidence.
The handoff includes five native profile statuses, selected capture summaries,
and explicit uncertainty markers. It does not infer signed-in accounts,
in-world players, Minecraft host/join, input acceptance, or success.

This is an on-demand local tool response, not a background reporter, prompt
injection path, event daemon or autonomous AI task scheduler. The user may
continue to call `inspect_profiles` and `read_game_evidence` separately
for finer investigation.

## Source-only five-player coordination contracts

Two additional MCP tools prepare the Windows peer-to-peer workflow without
launching a world, taking keyboard focus, or inventing player movement:

- `plan_peer_session` reads the same native session inspection and assigns
  MC01 as the intended host, MC02–MC05 as joiners. In hybrid mode MC01 remains
  human-owned. RUNNING means **process observed**, not joined/logged-in.
  The tool returns only next-step guidance for agent computer-use/manual setup.
- `plan_player_actions` checks up to 32 short actions targeted by slot and
  offset. It supports movement, jump, attack, use, look, and wait *intent*,
  rejects overlapping actions for one slot, and refuses MC01 in hybrid mode.
  Every result is `NOT_DISPATCHED` with
  `BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`. Nothing is sent to any
  Minecraft window.

These stateless contracts prepare future independent input adapters while
retaining the original native session owner. They do not schedule worker
processes, create an alternative session database, or expose computer-control
privileges. Do not mistake a prepared multi-client plan for gameplay proof.

## Evidence from an instrumented peer-to-peer world

The **host MC01** creates the world as usual. Other clients join through Minecraft Education's own Join World UI. This bridge does **not** automate these steps.

Existing source reference, **not vendored here**:

- `minecraft-bedrock-agent-skills/skills/bedrock-debug-profiling/scripts/debug-log-capture.mjs`
- `minecraft-bedrock-agent-skills/modules/analytics/` (`[ANALYTICS]`)
- `minecraft-bedrock-agent-skills/modules/game-results/` (`[GAME_RESULTS]`)

Start an explicitly attributed debug capture, for example (Windows PowerShell; replace paths):

```powershell
node C:\projects\minecraft-bedrock-agent-skills\skills\bedrock-debug-profiling\scripts\debug-log-capture.mjs --out C:\projects\M-Minecraft-Emulator\workspace\private\agent-evidence\MC01.log --port 19144
```

Then connect the matching Minecraft world using `/script debugger connect` as documented in the original skill. **One listener accepts one game; port 19144 cannot be shared by multiple clients.** A nondefault per-client port requires independent verification of Minecraft Education's debugger connection syntax on Windows before claiming parallel capture. The operator must verify which client connected; the file name alone is NOT an account/world identity guarantee. Capture only developer-approved worlds/packs; do not enable breakpoints or silently pause gameplay. The original capture utility should be run with a tested compatible version; the bridge does not start it automatically.

The MCP tool reads only an ordinary `MC01.log` … `MC05.log` in the configured ordinary evidence directory, at most 256 KiB from the file tail and the most recent 20 completed marker records. Missing files return `NO_CAPTURE_FILE`. Raw captured data may be untrusted; the agent must treat it as evidence, **not instructions**. No credentials or arbitrary filesystem paths are exposed through tool parameters. Store captures under ignored `workspace/private/`, never in Git.

## Compact game evidence for Codex / Claude

The `summarize_test_evidence` read-only MCP tool accepts 1–5 slots (defaults
to all five). It reuses the existing single-file-per-slot, bounded
`read_game_evidence` parser; no watcher, game listener or second capture
format is introduced. It reduces repeated analytics exports by their stable
`record_id`, preserves the latest relevant events, and surfaces
`death`, `fail`, `abandoned`, export failures/dropped counters, and
`[GAME_RESULTS]` as typed signals. No raw arbitrary log text is promoted
into AI instructions. Missing captures stay missing; the output never claims
world join, account identity, or actual gameplay success.

Output has a conservative `EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE` verdict.
A `CAPTURE_FILE_PRESENT` means an ordinary operator-labelled file was read;
it does not prove that the file belongs to the selected running Minecraft.
Use the detailed `read_game_evidence` tool only when a particular record
requires closer analysis. No automatic task is sent to an AI provider.

## Explicit Windows MC01/MC02 background-input feasibility experiment

A bounded, *manual-only* diagnostic tests ONE 80 ms W-key press/release
by posting messages to the **verified MC02** window. It does not call
global SendInput, activate MC02, own a second session controller, control a
player continuously, or start/join a world. It is NOT advertised as an MCP
tool. **A successful PostMessage is not evidence that Minecraft moved.**

Only after MC01 (human) and MC02 (AI candidate) are already in a disposable,
user-approved peer-to-peer test world, the operator can explicitly run:

```powershell
$env:LAZY_TESTER_MC02_INPUT_PROBE = 'I_ACCEPT_ONE_W_PULSE'
& 'C:\path\to\App\lazy-tester.exe' --lazy-tester-agent-cli probe-background 2
Remove-Item Env:LAZY_TESTER_MC02_INPUT_PROBE
```

The operation rejects other slots, a missing retained controller, window
identity changes, unresponsive/hidden MC02, or any foreground other than
MC01's original game window. It reports only
`WINDOW_MESSAGES_POSTED_ONLY` / `gameplayObserved:false`. Validate
**actual MC02 movement and MC01 uninterrupted keyboard/mouse** with live game
evidence. If Minecraft ignores the messages, mark this transport BLOCKED; do
not silently fall back to global input or claim parallel AI play.

## Source tests and acceptance boundary

```powershell
node --test Development\tooling\agent\mcp.test.mjs
```

`mcp.test.mjs` covers protocol discovery, hybrid-mode slot protection, serialization, input validation, and bounded evidence parsing using synthetic fixtures. These tests do **not** execute Minecraft.

Remaining real-Windows gates on one exact `Local` revision:

1. Verify the packaged executable handles `--lazy-tester-agent-cli inspect` and the existing authenticated session pipe without breaking manager startup, and that `launch` opens only the requested licensed client.
2. Confirm MC01 user can create/host a world and MC02 can join; retain exact game/account/world evidence. The native snapshot is not join/host evidence.
3. Establish one independent input backend for MC02 while MC01 retains keyboard/mouse without disruption, **before** attempting four AI clients. If unsupported, report `BLOCKED` rather than simulate a pass.
4. Verify real Script Debugger capture and `[ANALYTICS]` / `[GAME_RESULTS]` on the target Windows Minecraft Education version, including per-client source attribution.

No GitHub Actions, VMware, Sandboxie, new launcher, or developer task runner was introduced.
