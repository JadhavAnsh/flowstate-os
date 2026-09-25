import { useCallback, useEffect, useRef, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { type Event } from "@flowstate/protocol"
import { ensureActiveConversation } from "./shared/conversation"
import { useFlowStateEvents } from "./hooks/useFlowStateEvents"
import { useSpeechOutput } from "./hooks/useSpeechOutput"
import { useCommandDoubleTap } from "./hooks/useCommandDoubleTap"
import "./Hud.css"

export default function HudApp() {
  useCommandDoubleTap()
  const [revealed, setRevealed] = useState(false)
  const [conversationId, setConversationId] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [assistantStream, setAssistantStream] = useState("")
  const [expanded, setExpanded] = useState(false)
  const [awaitingResponse, setAwaitingResponse] = useState(false)
  const { push, finish, cancel: cancelSpeech } = useSpeechOutput(setError)
  const assistantStreamRef = useRef("")
  const holdTimerRef = useRef<number | null>(null)
  const captureStartedRef = useRef(false)
  const suppressClickRef = useRef(false)

  useEffect(() => {
    let disposed = false
    let unlisten: (() => void) | undefined
    let frame = 0
    let revision = 0
    const reveal = (visible: boolean) => {
      cancelAnimationFrame(frame)
      if (!visible) {
        setRevealed(false)
        return
      }
      // Give a newly shown native window one painted collapsed frame.
      frame = requestAnimationFrame(() => {
        frame = requestAnimationFrame(() => setRevealed(true))
      })
    }
    void (async () => {
      unlisten = await listen<boolean>("hud-reveal", ({ payload }) => {
        revision++
        if (!disposed) reveal(payload)
      })
      if (disposed) {
        unlisten()
        return
      }
      const before = revision
      const visible = await invoke<boolean>("hud_ready")
      if (!disposed && before === revision) reveal(visible)
    })().catch((err) => {
      if (!disposed) setError(String(err))
    })
    return () => {
      disposed = true
      unlisten?.()
      cancelAnimationFrame(frame)
    }
  }, [])

  const handleLiveEvent = useCallback(
    (event: Event) => {
      if (event.type === "model.delta") {
        if ((event.payload as { source?: string }).source !== "voice") return
        const delta = (event.payload as { delta?: string }).delta ?? ""
        assistantStreamRef.current += delta
        push(delta)
        setAssistantStream(assistantStreamRef.current)
      } else if (event.type === "model.completed") {
        if ((event.payload as { source?: string }).source !== "voice") return
        setAwaitingResponse(false)
        const text = (event.payload as { text?: string }).text
        finish()
        setAssistantStream(text || assistantStreamRef.current)
        assistantStreamRef.current = ""
      } else if (event.type === "permission.requested") {
        if ((event.payload as { source?: string }).source !== "voice") return
        setAwaitingResponse(false)
        finish()
        setAssistantStream("Cloud approval is waiting in the dashboard.")
      } else if (event.type === "task.failed") {
        setAwaitingResponse(false)
        assistantStreamRef.current = ""
        setAssistantStream("")
      }
    },
    [finish, push]
  )
  useFlowStateEvents(handleLiveEvent)

  const loadWidget = useCallback(async () => {
    setError(null)
    try {
      const active = await ensureActiveConversation()
      setConversationId(active)
    } catch (err) {
      setError(`Widget unavailable: ${String(err)}`)
    }
  }, [])

  useEffect(() => {
    void loadWidget()
  }, [loadWidget])

  async function openDashboard() {
    await invoke("focus_main_window", { conversationId })
  }
  const beginHold = useCallback(() => {
    if (!conversationId || holdTimerRef.current !== null) return
    suppressClickRef.current = false
    holdTimerRef.current = window.setTimeout(() => {
      holdTimerRef.current = null
      captureStartedRef.current = true
      suppressClickRef.current = true
      setError(null)
      setAssistantStream("")
      assistantStreamRef.current = ""
      cancelSpeech()
      void invoke("start_voice_capture", { conversationId })
        .then(() => setAwaitingResponse(true))
        .catch((reason) => {
          captureStartedRef.current = false
          setAwaitingResponse(false)
          setError(String(reason))
        })
    }, 180)
  }, [cancelSpeech, conversationId])

  const endHold = useCallback((cancel = false) => {
    if (holdTimerRef.current !== null) {
      window.clearTimeout(holdTimerRef.current)
      holdTimerRef.current = null
      return
    }
    if (!captureStartedRef.current) return
    captureStartedRef.current = false
    suppressClickRef.current = true
    void invoke(cancel ? "cancel_voice_capture" : "stop_voice_capture").catch(
      (reason) => {
        setAwaitingResponse(false)
        setError(String(reason))
      }
    )
  }, [])
  const hideWidget = useCallback(async () => {
    await invoke("hide_hud")
  }, [])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault()
        void hideWidget()
      }
    }
    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [hideWidget])

  const displayText = error || assistantStream
  const surface =
    expanded && displayText ? "expanded" : displayText ? "response" : "compact"
  const thinking = awaitingResponse && !assistantStream
  useEffect(() => {
    void invoke("set_hud_surface", { surface }).catch(() => undefined)
  }, [surface])

  const active = awaitingResponse
  useEffect(() => {
    void invoke("set_hud_busy", { busy: active }).catch(() => undefined)
  }, [active])

  return (
    <main
      className="hud-stage"
      data-revealed={revealed}
      data-surface={surface}
      data-thinking={thinking}
    >
      <section
        className={`lens ${active ? "is-active" : ""} ${error ? "has-error" : ""}`}
        aria-live="polite"
        aria-hidden={!revealed}
        inert={!revealed}
      >
        <button
          type="button"
          className="flow-orb"
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId)
            beginHold()
          }}
          onPointerUp={() => endHold(false)}
          onPointerCancel={() => endHold(true)}
          onClick={() => {
            if (suppressClickRef.current) {
              suppressClickRef.current = false
              return
            }
            void openDashboard()
          }}
          aria-label="Hold to talk. Tap to open FlowState dashboard."
        >
          <span />
        </button>
        <div className="waveform" aria-hidden="true">
          {[0, 1, 2, 3].map((bar) => (
            <i key={bar} />
          ))}
        </div>
        <span className="sr-only" role="status">
          {error
            ? "Flow needs attention"
            : thinking
              ? "Thinking"
              : "FlowState ready"}
        </span>
        {displayText && (
          <button
            type="button"
            className="lens-response"
            aria-label={expanded ? "Collapse response" : "Expand response"}
            aria-expanded={expanded}
            onClick={() => setExpanded(!expanded)}
          >
            {displayText}
          </button>
        )}
        {error ? (
          <button
            type="button"
            className="retry"
            onClick={() => void loadWidget()}
          >
            Try again
          </button>
        ) : null}
      </section>
    </main>
  )
}
