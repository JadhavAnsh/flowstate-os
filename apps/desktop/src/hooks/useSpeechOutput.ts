import { useCallback, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"

export function useSpeechOutput(onError?: (message: string) => void) {
  const onErrorRef = useRef(onError)
  onErrorRef.current = onError

  const cancel = useCallback(() => {
    void invoke("cancel_speech").catch((error) =>
      onErrorRef.current?.(String(error))
    )
  }, [])

  const speak = useCallback((text: string) => {
    if (!text.trim()) return
    void invoke("speak_text", { text }).catch((error) =>
      onErrorRef.current?.(String(error))
    )
  }, [])

  return { cancel, speak }
}
