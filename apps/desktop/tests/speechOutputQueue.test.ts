import { expect, test } from "bun:test"
import { SpeechOutputQueue } from "../src/shared/speechOutputQueue"

function deferred() {
  let resolve!: () => void
  const promise = new Promise<void>((done) => {
    resolve = done
  })
  return { promise, resolve }
}

const nextTurn = () => new Promise((resolve) => setTimeout(resolve, 0))

test("finish waits for the last native utterance to finish playing", async () => {
  const first = deferred()
  const last = deferred()
  const spoken: string[] = []
  const queue = new SpeechOutputQueue(
    {
      speak: async (text) => {
        spoken.push(text)
        await (spoken.length === 1 ? first.promise : last.promise)
      },
      cancel: async () => {},
    },
    () => {}
  )
  queue.push("This first sentence is long enough to start streaming. ")
  let completed = false
  const playback = Promise.resolve(
    queue.finish(
      "This first sentence is long enough to start streaming. Final words."
    )
  ).then((result) => {
    completed = true
    return result
  })
  await nextTurn()
  expect(spoken).toHaveLength(1)
  expect(completed).toBe(false)
  first.resolve()
  await nextTurn()
  expect(spoken).toEqual([
    "This first sentence is long enough to start streaming.",
    "Final words.",
  ])
  expect(completed).toBe(false)
  last.resolve()
  expect(await playback).toBe("finished")
})

test("barge-in interrupts active playback before waiting for its completion", async () => {
  const oldPlayback = deferred()
  const cancellation = deferred()
  const newPlayback = deferred()
  const calls: string[] = []
  const queue = new SpeechOutputQueue(
    {
      speak: async (text) => {
        calls.push(text)
        await (text === "Old response."
          ? oldPlayback.promise
          : newPlayback.promise)
      },
      cancel: async () => {
        calls.push("cancel")
        oldPlayback.resolve()
        await cancellation.promise
      },
    },
    () => {}
  )
  const previous = queue.finish("Old response.")
  await nextTurn()
  queue.cancel()
  const current = queue.finish("New response.")
  await nextTurn()
  expect(calls).toEqual(["Old response.", "cancel"])
  expect(await previous).toBe("cancelled")
  cancellation.resolve()
  await nextTurn()
  expect(calls).toEqual(["Old response.", "cancel", "New response."])
  newPlayback.resolve()
  expect(await current).toBe("finished")
})

test("speaks completion-only responses and reconciles a missing final delta", async () => {
  const spoken: string[] = []
  const queue = new SpeechOutputQueue(
    {
      speak: async (text) => {
        spoken.push(text)
      },
      cancel: async () => {},
    },
    () => {}
  )
  queue.finish("Hello from FlowState.")
  queue.push("This response has a complete sentence ready for speech. ")
  queue.finish(
    "This response has a complete sentence ready for speech. And its final tail."
  )
  await queue.settled()
  expect(spoken).toEqual([
    "Hello from FlowState.",
    "This response has a complete sentence ready for speech.",
    "And its final tail.",
  ])
})

test("barge-in drops queued old sentences and waits for native cancellation", async () => {
  const calls: string[] = []
  let completeCancel!: () => void
  const cancelled = new Promise<void>((resolve) => {
    completeCancel = resolve
  })
  const queue = new SpeechOutputQueue(
    {
      speak: async (text) => {
        calls.push(text)
      },
      cancel: async () => {
        calls.push("cancel")
        await cancelled
      },
    },
    () => {}
  )
  queue.finish("This old sentence must never reach the native speech queue.")
  queue.cancel()
  queue.finish("New response.")
  await new Promise((resolve) => setTimeout(resolve, 0))
  expect(calls).toEqual(["cancel"])
  completeCancel()
  await queue.settled()
  expect(calls).toEqual(["cancel", "New response."])
})

test("native errors remain readable and do not break later playback", async () => {
  const errors: string[] = []
  const spoken: string[] = []
  const queue = new SpeechOutputQueue(
    {
      speak: async (text) => {
        spoken.push(text)
      },
      cancel: async () => {
        throw { message: "Audio unavailable" }
      },
    },
    (error) => errors.push(error)
  )
  queue.speak("Try again.")
  await queue.settled()
  expect(errors).toEqual(["Audio unavailable"])
  expect(spoken).toEqual(["Try again."])
})

test("interrupted native errors cannot repopulate the next turn", async () => {
  let failOldSpeech!: (error: unknown) => void
  const oldSpeech = new Promise<void>((_, reject) => {
    failOldSpeech = reject
  })
  const errors: string[] = []
  const spoken: string[] = []
  const queue = new SpeechOutputQueue(
    {
      speak: async (text) => {
        spoken.push(text)
        if (text === "Old response.") await oldSpeech
      },
      cancel: async () => {
        failOldSpeech({ message: "Old audio was interrupted" })
      },
    },
    (message) => errors.push(message)
  )
  const previous = queue.finish("Old response.")
  await nextTurn()
  queue.push("Old unfinished words")
  queue.cancel()
  expect(await queue.finish("New response.")).toBe("finished")
  expect(await previous).toBe("cancelled")
  expect(spoken).toEqual(["Old response.", "New response."])
  expect(errors).toEqual([])
})

test("failed output reports an error and settles instead of leaving the widget busy", async () => {
  const errors: string[] = []
  const queue = new SpeechOutputQueue(
    {
      speak: async () => {
        throw { message: "Audio device unavailable" }
      },
      cancel: async () => {},
    },
    (message) => errors.push(message)
  )
  expect(await queue.finish("Response.")).toBe("finished")
  expect(errors).toEqual(["Audio device unavailable"])
})
