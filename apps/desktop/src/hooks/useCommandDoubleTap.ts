import { useEffect, useRef } from "react"
import { invoke } from "@tauri-apps/api/core"

const DOUBLE_TAP_MS = 450

/** Focused-window fallback. Reveal is idempotent with the native macOS monitor. */
export function useCommandDoubleTap() {
  const lastTapRef = useRef(0)
  const chordActiveRef = useRef(false)

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.ctrlKey && event.altKey && !chordActiveRef.current) {
        event.preventDefault()
        chordActiveRef.current = true
        void invoke("show_hud").catch(() => undefined)
        return
      }
      if (
        event.key !== "Meta" &&
        event.code !== "MetaLeft" &&
        event.code !== "MetaRight"
      ) {
        return
      }
      if (event.repeat) {
        return
      }
      const now = Date.now()
      if (now - lastTapRef.current <= DOUBLE_TAP_MS) {
        event.preventDefault()
        lastTapRef.current = 0
        void invoke("show_hud").catch(() => undefined)
        return
      }
      lastTapRef.current = now
    }

    const onKeyUp = (event: KeyboardEvent) => {
      if (event.key === "Control" || event.key === "Alt") {
        chordActiveRef.current = false
      }
    }

    const reset = () => {
      lastTapRef.current = 0
      chordActiveRef.current = false
    }
    window.addEventListener("blur", reset)
    window.addEventListener("keydown", onKeyDown, true)
    window.addEventListener("keyup", onKeyUp, true)
    return () => {
      window.removeEventListener("blur", reset)
      window.removeEventListener("keydown", onKeyDown, true)
      window.removeEventListener("keyup", onKeyUp, true)
    }
  }, [])
}
