import { expect, test } from "bun:test"
import { takeStableSentences } from "../src/shared/speechChunks"

test("holds an unstable tail until completion", () => {
  const first = takeStableSentences("This answer is still too short.")
  expect(first.chunks).toEqual([])
  expect(first.remaining).toContain("still too short")
  expect(takeStableSentences(first.remaining, true).chunks).toEqual([
    "This answer is still too short.",
  ])
})

test("emits complete sentences between 40 and 180 characters", () => {
  const text =
    "This is a complete sentence that is ready to be spoken. A tiny tail"
  const result = takeStableSentences(text)
  expect(result.chunks).toEqual([
    "This is a complete sentence that is ready to be spoken.",
  ])
  expect(result.remaining).toBe("A tiny tail")
})

test("hard-wraps long text without exceeding 180 characters", () => {
  const result = takeStableSentences("word ".repeat(50))
  expect(result.chunks[0].length).toBeLessThanOrEqual(180)
  expect(result.chunks[0].length).toBeGreaterThanOrEqual(40)
})
