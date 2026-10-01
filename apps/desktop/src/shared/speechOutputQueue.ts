import { takeStableSentences } from "./speechChunks"

export interface SpeechTransport {
  /** Resolves only after this utterance finishes or is interrupted. */
  speak(text: string): Promise<unknown>
  cancel(): Promise<unknown>
}

export type SpeechCompletion = "finished" | "cancelled"

export function voiceErrorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    return String(error.message)
  }
  return String(error)
}

/** Keep cancellation and sentences ordered across asynchronous native IPC. */
export class SpeechOutputQueue {
  private pending: Promise<void> = Promise.resolve()
  private revision = 0
  private buffer = ""
  private streamed = ""

  constructor(
    private readonly transport: SpeechTransport,
    private readonly onError: (message: string) => void
  ) {}

  private enqueue(text: string) {
    const revision = this.revision
    this.pending = this.pending
      .then(async () => {
        if (revision === this.revision) await this.transport.speak(text)
      })
      .catch((error) => {
        if (revision === this.revision) this.onError(voiceErrorMessage(error))
      })
  }

  cancel() {
    const revision = ++this.revision
    this.buffer = ""
    this.streamed = ""
    // Playback now settles at audio completion, so cancellation must bypass it.
    // New utterances still wait for both the interrupted audio and cancellation.
    const cancellation = Promise.resolve()
      .then(async () => {
        await this.transport.cancel()
      })
      .catch((error) => {
        if (revision === this.revision) this.onError(voiceErrorMessage(error))
      })
    this.pending = Promise.all([this.pending, cancellation]).then(() => {})
    return this.pending
  }

  push(delta: string) {
    this.streamed += delta
    this.buffer += delta
    const result = takeStableSentences(this.buffer)
    this.buffer = result.remaining
    result.chunks.forEach((text) => this.enqueue(text))
  }

  finish(finalText?: string): Promise<SpeechCompletion> {
    const revision = this.revision
    // Completion can race the last streamed delta. Reconcile the missing tail
    // without speaking already-queued sentences a second time.
    if (finalText?.startsWith(this.streamed)) {
      this.push(finalText.slice(this.streamed.length))
    }
    const result = takeStableSentences(this.buffer, true)
    this.buffer = ""
    this.streamed = ""
    result.chunks.forEach((text) => this.enqueue(text))
    return this.pending.then(() =>
      revision === this.revision ? "finished" : "cancelled"
    )
  }

  speak(text: string) {
    this.cancel()
    return this.finish(text)
  }

  settled() {
    return this.pending
  }
}
