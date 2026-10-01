/**
 * Browser regression for the real HudApp, with native IPC at a controlled seam.
 * Start `bun --cwd apps/desktop dev`, then run this script with Node.
 * Optional: HUD_URL, PLAYWRIGHT_MODULE, CHROME_EXECUTABLE, HUD_SCREENSHOT_PATH.
 * BASELINE_HUD_MODULE can point to a saved Vite-transformed HudApp module.
 * This verifies rendering/event lifecycle, not physical keys, TCC, or audible TTS.
 */
import assert from "node:assert/strict"
import { readFile } from "node:fs/promises"
import { homedir } from "node:os"
import { pathToFileURL } from "node:url"

const playwrightModule =
  process.env.PLAYWRIGHT_MODULE ??
  `${homedir()}/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright/index.mjs`
const { chromium } = await import(pathToFileURL(playwrightModule).href)
const browser = await chromium.launch({
  executablePath:
    process.env.CHROME_EXECUTABLE ??
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  headless: true,
})
const failures = []

async function openHud() {
  const page = await browser.newPage({ viewport: { width: 760, height: 440 } })
  const errors = []
  page.on("pageerror", (error) => errors.push(error.message))
  if (process.env.BASELINE_HUD_MODULE) {
    const body = await readFile(process.env.BASELINE_HUD_MODULE, "utf8")
    await page.route("**/src/HudApp.tsx*", (route) =>
      route.fulfill({
        contentType: "text/javascript",
        body,
      })
    )
  }
  await page.addInitScript(() => {
    const callbacks = new Map()
    const listeners = new Map()
    const calls = []
    const speech = []
    let nextId = 1
    let nextEventId = 1
    let failStart = false
    let deferStart = false
    let completeStart
    const emit = (event, payload) => {
      for (const [id, entry] of [...listeners]) {
        if (entry.event === event)
          callbacks.get(entry.handler)?.({ event, id, payload })
      }
    }
    const protocol = (type, payload, runId) =>
      emit("flowstate-event", {
        id: `event-${nextEventId++}`,
        schemaVersion: 1,
        occurredAt: new Date().toISOString(),
        type,
        payload,
        ...(runId ? { runId } : {}),
      })
    window.__hudTest = {
      calls,
      speech,
      emit,
      protocol,
      get listeners() {
        return [...listeners.values()].map((entry) => entry.event)
      },
      failNextStart() {
        failStart = true
      },
      deferNextStart() {
        deferStart = true
      },
      completeStart() {
        completeStart?.()
      },
      completeSpeech(index = 0) {
        const item = speech[index]
        if (!item) throw new Error(`No pending speech at index ${index}`)
        item.resolve()
        item.settled = true
      },
    }
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      unregisterListener: (_event, id) => listeners.delete(id),
    }
    window.__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: "hud" },
        currentWebview: { label: "hud" },
      },
      transformCallback(callback) {
        const id = nextId++
        callbacks.set(id, callback)
        return id
      },
      unregisterCallback: (id) => callbacks.delete(id),
      async invoke(command, args = {}) {
        calls.push({ command, args })
        switch (command) {
          case "plugin:event|listen": {
            const id = nextId++
            listeners.set(id, args)
            return id
          }
          case "plugin:event|unlisten":
            listeners.delete(args.eventId)
            return
          case "hud_ready":
            calls.at(-1).listeners = [...listeners.values()].map(
              (entry) => entry.event
            )
            return false
          case "list_conversations":
            return [{ id: "conversation-hud", title: "Main" }]
          case "set_active_conversation":
          case "set_hud_surface":
          case "set_hud_busy":
          case "focus_main_window":
            return
          case "start_voice_capture":
            if (failStart) {
              failStart = false
              throw new Error("Microphone temporarily unavailable")
            }
            if (deferStart) {
              deferStart = false
              await new Promise((resolve) => {
                completeStart = resolve
              })
            }
            return
          case "stop_voice_capture":
            return { status: "processing" }
          case "cancel_voice_capture":
            return null
          case "speak_text":
            return new Promise((resolve) =>
              speech.push({ text: args.text, resolve, settled: false })
            )
          case "cancel_speech":
            for (const item of speech) {
              if (!item.settled) {
                item.settled = true
                item.resolve()
              }
            }
            return
          case "hide_hud":
            emit("hud-reveal", false)
            return
          case "show_hud":
            emit("hud-reveal", true)
            return
          default:
            throw new Error(`Unmocked native command: ${command}`)
        }
      },
    }
  })
  await page.goto(process.env.HUD_URL ?? "http://127.0.0.1:1420/")
  await page.waitForFunction(() => {
    const test = window.__hudTest
    return (
      document.querySelector(".hud-stage") &&
      ["hud-reveal", "hud-ptt", "flowstate-event"].every((name) =>
        test.listeners.includes(name)
      ) &&
      test.calls.some((call) => call.command === "set_active_conversation")
    )
  })
  assert.deepEqual(errors, [], "HUD should mount without JavaScript errors")
  return page
}

async function emit(page, event, payload) {
  await page.evaluate(
    ([name, value]) => window.__hudTest.emit(name, value),
    [event, payload]
  )
}
async function protocol(page, type, payload, runId) {
  await page.evaluate(
    ([type, payload, runId]) => window.__hudTest.protocol(type, payload, runId),
    [type, payload, runId]
  )
}
async function transcript(page, text, final = false, extra = {}) {
  await protocol(page, "agent.message", {
    source: "hud",
    conversationId: "conversation-hud",
    text,
    partial: !final,
    kind: final ? "transcript.final" : "transcript.partial",
    ...extra,
  })
}
async function voice(page, type, runId, extra = {}) {
  await protocol(
    page,
    type,
    { source: "voice", conversationId: "conversation-hud", ...extra },
    runId
  )
}
async function reveal(page) {
  await emit(page, "hud-reveal", true)
  await page.waitForFunction(
    () => document.querySelector(".hud-stage")?.dataset.revealed === "true"
  )
}
async function hold(page) {
  const before = await callCount(page, "start_voice_capture")
  await reveal(page)
  await emit(page, "hud-ptt", true)
  await page.waitForFunction(
    (before) =>
      window.__hudTest.calls.filter(
        (call) => call.command === "start_voice_capture"
      ).length > before,
    before
  )
}
async function release(page) {
  const before = await callCount(page, "stop_voice_capture")
  await emit(page, "hud-ptt", false)
  await page.waitForFunction(
    (before) =>
      window.__hudTest.calls.filter(
        (call) => call.command === "stop_voice_capture"
      ).length > before,
    before
  )
}
async function callCount(page, command) {
  return page.evaluate(
    (command) =>
      window.__hudTest.calls.filter((call) => call.command === command).length,
    command
  )
}
async function response(page, expected) {
  await page.waitForFunction(
    (expected) =>
      document.querySelector(".lens-response")?.textContent === expected,
    expected,
    { timeout: 1500 }
  )
}
async function assertClean(page) {
  await page.waitForFunction(
    () => {
      const stage = document.querySelector(".hud-stage")
      return (
        stage?.dataset.revealed === "false" &&
        stage.dataset.surface === "compact" &&
        !document.querySelector(".lens-response") &&
        !document.querySelector(".has-error")
      )
    },
    undefined,
    { timeout: 1500 }
  )
}
async function run(name, test) {
  let page
  try {
    page = await openHud()
    await test(page)
    console.log(`PASS ${name}`)
  } catch (error) {
    failures.push(name)
    console.error(`FAIL ${name}: ${error.message.split("\n")[0]}`)
    if (page)
      console.error(
        await page.locator(".hud-stage").evaluate((el) => el.outerHTML)
      )
  } finally {
    await page?.close()
  }
}

try {
  await run(
    "all HUD listeners are installed before the native ready handshake",
    async (page) => {
      const readyCalls = await page.evaluate(() =>
        window.__hudTest.calls.filter((call) => call.command === "hud_ready")
      )
      assert.ok(readyCalls.length > 0)
      for (const call of readyCalls) {
        for (const event of ["hud-reveal", "hud-ptt", "flowstate-event"]) {
          assert.ok(
            call.listeners.includes(event),
            `${event} missing when hud_ready ran`
          )
        }
      }
    }
  )

  await run(
    "held keys reveal live partial and final transcription in the widget",
    async (page) => {
      await hold(page)
      await transcript(page, "Show my next meeting")
      await response(page, "Show my next meeting")
      await transcript(page, "Show my next meeting tomorrow", true)
      await response(page, "Show my next meeting tomorrow")
      assert.equal(
        await page.locator(".hud-stage").getAttribute("data-revealed"),
        "true"
      )
      if (process.env.HUD_SCREENSHOT_PATH)
        await page.screenshot({ path: process.env.HUD_SCREENSHOT_PATH })
    }
  )

  await run(
    "widget remains visible until native speech finishes, then clears all turn state",
    async (page) => {
      await hold(page)
      await release(page)
      await voice(page, "model.started", "run-output")
      await voice(page, "model.delta", "run-output", {
        delta: "Your meeting is tomorrow.",
      })
      await voice(page, "model.completed", "run-output", {
        text: "Your meeting is tomorrow.",
      })
      await response(page, "Your meeting is tomorrow.")
      await page.locator(".lens-response").click()
      assert.equal(
        await page.locator(".hud-stage").getAttribute("data-surface"),
        "expanded"
      )
      await page.waitForFunction(() => window.__hudTest.speech.length === 1)
      await page.waitForTimeout(2100)
      assert.equal(
        await page.locator(".hud-stage").getAttribute("data-revealed"),
        "true",
        "model completion/timer must not hide pending native speech"
      )
      assert.equal(await callCount(page, "hide_hud"), 0)
      await page.evaluate(() => window.__hudTest.completeSpeech())
      await assertClean(page)
      await reveal(page)
      assert.equal(
        await page.locator(".lens-response").count(),
        0,
        "reopening must not restore previous content"
      )
      assert.equal(
        await page.locator(".hud-stage").getAttribute("data-surface"),
        "compact"
      )
    }
  )

  await run(
    "collapse clears errors and ignores stale events while closed or idle",
    async (page) => {
      await page.evaluate(() => window.__hudTest.failNextStart())
      await hold(page)
      await page.waitForFunction(() => document.querySelector(".has-error"))
      await emit(page, "hud-reveal", false)
      await assertClean(page)
      await transcript(page, "Stale transcript")
      await voice(page, "model.started", "run-stale")
      await voice(page, "model.delta", "run-stale", {
        delta: "Stale response.",
      })
      await voice(page, "model.completed", "run-stale", {
        text: "Stale response.",
      })
      await reveal(page)
      await transcript(page, "Another stale transcript")
      assert.equal(await page.locator(".lens-response").count(), 0)
      assert.equal(await page.locator(".has-error").count(), 0)
      assert.equal(await callCount(page, "speak_text"), 0)
    }
  )

  await run(
    "unrelated conversations and desktop messages never replace live HUD text",
    async (page) => {
      await hold(page)
      await transcript(page, "Current words")
      await response(page, "Current words")
      await transcript(page, "Other conversation", false, {
        conversationId: "conversation-other",
      })
      await transcript(page, "Dashboard text", false, { source: "desktop" })
      await voice(page, "model.delta", "run-other", {
        conversationId: "conversation-other",
        delta: "Wrong output.",
      })
      await response(page, "Current words")
    }
  )

  await run(
    "a new held turn cannot be collapsed by the previous speech completion",
    async (page) => {
      await hold(page)
      await release(page)
      await voice(page, "model.started", "run-old")
      await voice(page, "model.delta", "run-old", {
        delta: "Previous response.",
      })
      await voice(page, "model.completed", "run-old", {
        text: "Previous response.",
      })
      await page.waitForFunction(() => window.__hudTest.speech.length === 1)
      await hold(page)
      await transcript(page, "New live words")
      await response(page, "New live words")
      await page.waitForTimeout(50)
      assert.equal(
        await page.locator(".hud-stage").getAttribute("data-revealed"),
        "true"
      )
      await release(page)
      await voice(page, "model.started", "run-new")
      await voice(page, "model.completed", "run-old", {
        text: "Late previous response.",
      })
      await voice(page, "model.delta", "run-old", {
        delta: "Late previous delta.",
      })
      await voice(page, "model.delta", "run-new", { delta: "New response." })
      await voice(page, "model.completed", "run-new", { text: "New response." })
      await response(page, "New response.")
      await page.waitForFunction(() => window.__hudTest.speech.length === 2)
      await page.waitForTimeout(2100)
      assert.equal(
        await page.locator(".hud-stage").getAttribute("data-revealed"),
        "true"
      )
      assert.equal(await callCount(page, "hide_hud"), 0)
      await page.evaluate(() => window.__hudTest.completeSpeech(1))
      await assertClean(page)
    }
  )
} finally {
  await browser.close()
}

if (failures.length) {
  console.error(`HUD lifecycle: ${failures.length} scenario(s) failed.`)
  process.exitCode = 1
} else {
  console.log("HUD lifecycle: all 6 scenarios passed.")
}
