import { useCallback, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"
import { SpeechOutputQueue } from "../shared/speechOutputQueue"

export function useSpeechOutput(onError?: (message: string) => void) {
  const onErrorRef = useRef(onError)
  onErrorRef.current = onError
  const queueRef = useRef<SpeechOutputQueue | null>(null)
  if (!queueRef.current) {
    queueRef.current = new SpeechOutputQueue(
      {
        speak: (text) => invoke("speak_text", { text }),
        cancel: () => invoke("cancel_speech"),
      },
      (message) => onErrorRef.current?.(message)
    )
  }

  const cancel = useCallback(() => queueRef.current!.cancel(), [])
  const push = useCallback((delta: string) => queueRef.current!.push(delta), [])
  const finish = useCallback(
    (text?: string) => queueRef.current!.finish(text),
    []
  )
  const speak = useCallback((text: string) => queueRef.current!.speak(text), [])

  return { cancel, finish, push, speak }
}
