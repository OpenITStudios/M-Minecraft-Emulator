// Minecraft Education Code Builder > Microsoft MakeCode > JavaScript
// EXPERIMENTAL / NOT VERIFIED. Add the official "File Read & Write" extension.
// Use ONE generated copy per AI client. These paths must already exist.
// This is world-side teleportation, NOT true WASD, click, mining or PvP input.
// The local Node writer does not send physical keyboard/mouse input.
//
// Replace the two placeholders by renderMakeCodeTemplate(...) in file-relay.mjs.
let inputPath = __INPUT_FILE__
let ackPath = __ACK_FILE__
let lastSequence = 0

// Suppress stale-command replay if Code Builder restarts.
let boot = file.readFile(inputPath).split("|")
if (boot.length == 3 && boot[0] == "LT1") {
    let candidate = parseInt(boot[1])
    if (candidate >= 0 && candidate <= 999999) {
        lastSequence = candidate
    }
}
file.writeFile(ackPath, "LT1|" + lastSequence + "|READY")

loops.forever(function () {
    let msg = file.readFile(inputPath).split("|")
    if (msg.length == 3 && msg[0] == "LT1") {
        let seq = parseInt(msg[1])
        if (seq == lastSequence + 1 && seq > 0 && seq <= 999999) {
            lastSequence = seq
            let status = "UNSUPPORTED"
            if (msg[2] == "PING") {
                status = "PING_ACK"
            } else if (msg[2] == "STEP_NORTH") {
                status = player.teleport(positions.add(player.position(), pos(0, 0, -1)))
                    ? "STEP_OK" : "STEP_FAILED"
            } else if (msg[2] == "STEP_SOUTH") {
                status = player.teleport(positions.add(player.position(), pos(0, 0, 1)))
                    ? "STEP_OK" : "STEP_FAILED"
            } else if (msg[2] == "STEP_EAST") {
                status = player.teleport(positions.add(player.position(), pos(1, 0, 0)))
                    ? "STEP_OK" : "STEP_FAILED"
            } else if (msg[2] == "STEP_WEST") {
                status = player.teleport(positions.add(player.position(), pos(-1, 0, 0)))
                    ? "STEP_OK" : "STEP_FAILED"
            }
            file.writeFile(ackPath, "LT1|" + seq + "|" + status)
        }
    }
    loops.pause(250)
})
