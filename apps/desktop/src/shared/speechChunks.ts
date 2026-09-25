const MIN_SENTENCE = 40
const MAX_SENTENCE = 180

export function takeStableSentences(input: string, flush = false) {
  const chunks: string[] = []
  let remaining = input
  while (remaining.length) {
    const bounded = remaining.slice(0, MAX_SENTENCE + 1)
    const sentenceMatch = bounded.match(/^([\s\S]{40,180}?[.!?](?:\s|$))/)
    let cut = sentenceMatch?.[1].length ?? 0
    if (!cut && remaining.length > MAX_SENTENCE) {
      cut = remaining.slice(0, MAX_SENTENCE + 1).lastIndexOf(" ")
      if (cut < MIN_SENTENCE) cut = MAX_SENTENCE
    }
    if (!cut) break
    const chunk = remaining.slice(0, cut).trim()
    if (chunk) chunks.push(chunk)
    remaining = remaining.slice(cut).trimStart()
  }
  if (flush && remaining.trim()) {
    chunks.push(remaining.trim())
    remaining = ""
  }
  return { chunks, remaining }
}
