// Hardware smoke: emits test speech, checks delegate playback/cancellation, and
// keeps stdin OPEN throughout. Closing stdin masks a blocked main actor.
import assert from "node:assert/strict"
import { spawn } from "node:child_process"
import { createInterface } from "node:readline"
import { resolve } from "node:path"

const binary =
  process.argv[2] ??
  resolve(
    import.meta.dirname,
    "../native/apple-runtime/.build/release/flowstate-apple-runtime"
  )
const child = spawn(binary, [], { stdio: ["pipe", "pipe", "pipe"] })
const events = []
const output = createInterface({ input: child.stdout })
output.on("line", (line) => {
  try {
    events.push(JSON.parse(line))
  } catch {
    /* system output */
  }
})
child.stderr.resume()
let exited = false
child.on("exit", () => {
  exited = true
})
child.on("error", (error) => {
  console.error(error.message)
  exited = true
})
const send = (id, command, fields = {}) => {
  child.stdin.write(JSON.stringify({ id, command, ...fields }) + "\n")
}
async function waitFor(id, event, timeout = 5000) {
  const deadline = Date.now() + timeout
  while (Date.now() < deadline) {
    const match = events.find((item) => item.id === id && item.event === event)
    if (match) return match
    assert(!exited, `Helper exited while waiting for ${id}/${event}`)
    await new Promise((resolve) => setTimeout(resolve, 20))
  }
  assert.fail(
    `Timed out waiting for ${id}/${event} with stdin open. Received: ${events
      .map(({ id, event, code }) => `${id}/${event}${code ? `/${code}` : ""}`)
      .join(", ")}`
  )
}
try {
  send("readiness", "invalid_probe")
  await waitFor("readiness", "error")
  send("playback", "speak", {
    locale: "en-IN",
    text: "FlowState voice playback test. This sentence will be interrupted by the cancellation check.",
  })
  await waitFor("playback", "speech_started", 15000)
  assert(
    !events.some(({ id, event }) => id === "playback" && event === "result"),
    "Speak must remain pending while audio is playing"
  )
  send("cancel", "cancel_speech")
  await waitFor("cancel", "result")
  await waitFor("playback", "speech_cancelled")
  await waitFor("playback", "result")
  send("complete", "speak", {
    locale: "en-IN",
    text: "Voice output is working.",
  })
  await waitFor("complete", "speech_started")
  await waitFor("complete", "speech_finished", 10000)
  await waitFor("complete", "result")
  for (const [requestId, terminal] of [
    ["playback", "speech_cancelled"],
    ["complete", "speech_finished"],
  ]) {
    const replies = events.filter(
      ({ id, event }) => id === requestId && event === "result"
    )
    assert.equal(replies.length, 1, `${requestId} must finish exactly once`)
    const terminalIndex = events.findIndex(
      ({ id, event }) => id === requestId && event === terminal
    )
    const resultIndex = events.findIndex(
      ({ id, event }) => id === requestId && event === "result"
    )
    assert(
      resultIndex > terminalIndex,
      `${requestId} result must follow ${terminal}`
    )
  }
  send("empty", "speak", { text: "  " })
  await waitFor("empty", "result")
  send("exit", "shutdown")
  await waitFor("exit", "result")
  const deadline = Date.now() + 2000
  while (!exited && Date.now() < deadline)
    await new Promise((resolve) => setTimeout(resolve, 20))
  assert(exited, "Helper must exit without requiring stdin EOF")
  console.log(
    "PASS: speech replies followed playback/cancellation, empty speech settled, and helper shut down with stdin open."
  )
} finally {
  child.stdin.destroy()
  output.close()
  if (!exited) child.kill("SIGTERM")
}
