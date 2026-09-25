import { useCallback, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"
import { takeStableSentences } from "../shared/speechChunks"

export function useSpeechOutput(onError?: (message: string) => void) {
  const onErrorRef = useRef(onError)
  const bufferRef = useRef("")
  onErrorRef.current = onError

  const cancel = useCallback(() => {
    bufferRef.current = ""
    void invoke("cancel_speech").catch((error) =>
      onErrorRef.current?.(String(error))
    )
  }, [])

  const enqueue = useCallback((text: string) => {
    if (!text.trim()) return
    void invoke("speak_text", { text }).catch((error) =>
      onErrorRef.current?.(String(error))
    )
  }, [])

  const push = useCallback(
    (delta: string) => {
      bufferRef.current += delta
      const result = takeStableSentences(bufferRef.current)
      bufferRef.current = result.remaining
      result.chunks.forEach(enqueue)
    },
    [enqueue]
  )

  const finish = useCallback(() => {
    const result = takeStableSentences(bufferRef.current, true)
    bufferRef.current = ""
    result.chunks.forEach(enqueue)
  }, [enqueue])

  const speak = useCallback(
    (text: string) => {
      cancel()
      takeStableSentences(text, true).chunks.forEach(enqueue)
    },
    [cancel, enqueue]
  )

  return { cancel, finish, push, speak }
}
