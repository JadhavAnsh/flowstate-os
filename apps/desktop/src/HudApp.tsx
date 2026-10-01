import { useCallback, useEffect, useRef, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { parseProtocolRecord, type Event } from "@flowstate/protocol"
import { ensureActiveConversation } from "./shared/conversation"
import { useSpeechOutput } from "./hooks/useSpeechOutput"
import { useCommandDoubleTap } from "./hooks/useCommandDoubleTap"
import "./Hud.css"

type Phase = "idle" | "listening" | "thinking" | "responding" | "speaking"
type VoiceTurn = {
  generation: number
  conversationId: string | null
  runId: string | null
  phase: Phase
}

export default function HudApp() {
  useCommandDoubleTap()
  const [revealed, setRevealed] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [transcript, setTranscript] = useState("")
  const [assistantStream, setAssistantStream] = useState("")
  const [expanded, setExpanded] = useState(false)
  const [phase, setPhase] = useState<Phase>("idle")
  const { push, finish, cancel: cancelSpeech } = useSpeechOutput(setError)
  const conversationIdRef = useRef<string | null>(null)
  const generationRef = useRef(0)
  const turnRef = useRef<VoiceTurn | null>(null)
  const assistantStreamRef = useRef("")
  const holdTimerRef = useRef<number | null>(null)
  const captureStartedRef = useRef(false)
  const captureReadyRef = useRef(false)
  const releaseRequestedRef = useRef<"stop" | "cancel" | null>(null)
  const suppressClickRef = useRef(false)

  const clearWidget = useCallback(() => {
    generationRef.current++
    turnRef.current = null
    assistantStreamRef.current = ""
    setTranscript("")
    setAssistantStream("")
    setError(null)
    setExpanded(false)
    setPhase("idle")
  }, [])

  const collapse = useCallback(
    (generation: number) => {
      if (generation !== generationRef.current) return
      clearWidget()
      setRevealed(false)
      void invoke("hide_hud").catch((reason) => setError(String(reason)))
    },
    [clearWidget]
  )

  const failCapture = useCallback(
    (generation: number, reason: unknown) => {
      if (generation !== generationRef.current) return
      captureStartedRef.current = false
      captureReadyRef.current = false
      clearWidget()
      setError(String(reason))
    },
    [clearWidget]
  )

  const loadWidget = useCallback(async () => {
    setError(null)
    try {
      conversationIdRef.current = await ensureActiveConversation()
    } catch (reason) {
      setError(`Widget unavailable: ${String(reason)}`)
    }
  }, [])

  useEffect(() => {
    void loadWidget()
  }, [loadWidget])

  const finalizeCapture = useCallback(
    async (cancel = false) => {
      const turn = turnRef.current
      if (!captureStartedRef.current || !turn) return
      captureStartedRef.current = false
      captureReadyRef.current = false
      suppressClickRef.current = true
      turn.phase = "thinking"
      setPhase("thinking")
      try {
        const outcome = await invoke(
          cancel ? "cancel_voice_capture" : "stop_voice_capture"
        )
        if (cancel || outcome === null) collapse(turn.generation)
      } catch (reason) {
        failCapture(turn.generation, reason)
      }
    },
    [collapse, failCapture]
  )

  const startCapture = useCallback(async () => {
    if (captureStartedRef.current) return
    const generation = ++generationRef.current
    const turn: VoiceTurn = {
      generation,
      conversationId: conversationIdRef.current,
      runId: null,
      phase: "listening",
    }
    turnRef.current = turn
    captureStartedRef.current = true
    captureReadyRef.current = false
    releaseRequestedRef.current = null
    suppressClickRef.current = true
    setRevealed(true)
    setPhase("listening")
    setError(null)
    setTranscript("")
    setAssistantStream("")
    setExpanded(false)
    assistantStreamRef.current = ""
    void cancelSpeech()
    try {
      const conversationId =
        conversationIdRef.current ?? (await ensureActiveConversation())
      if (generation !== generationRef.current) {
        captureStartedRef.current = false
        return
      }
      conversationIdRef.current = conversationId
      turn.conversationId = conversationId
      if (releaseRequestedRef.current) {
        captureStartedRef.current = false
        collapse(generation)
        return
      }
      await invoke("start_voice_capture", { conversationId })
      if (generation !== generationRef.current) {
        // A dismissal during native startup must still stop the microphone.
        // Keep start guarded until cancellation finishes so it cannot cancel
        // capture belonging to a newer turn.
        try {
          await invoke("cancel_voice_capture")
        } finally {
          captureStartedRef.current = false
          captureReadyRef.current = false
        }
        return
      }
      captureReadyRef.current = true
      if (releaseRequestedRef.current) {
        await finalizeCapture(releaseRequestedRef.current === "cancel")
      }
    } catch (reason) {
      captureStartedRef.current = false
      captureReadyRef.current = false
      failCapture(generation, reason)
    }
  }, [cancelSpeech, collapse, failCapture, finalizeCapture])

  const stopCapture = useCallback(
    async (cancel = false) => {
      releaseRequestedRef.current = cancel ? "cancel" : "stop"
      if (!captureStartedRef.current || !captureReadyRef.current) return
      await finalizeCapture(cancel)
    },
    [finalizeCapture]
  )

  const handleLiveEvent = useCallback(
    (event: Event) => {
      const turn = turnRef.current
      if (!turn || event.payload.conversationId !== turn.conversationId) return
      const payload = event.payload
      if (event.type === "agent.message") {
        if (
          payload.source === "hud" &&
          (payload.kind === "transcript.partial" ||
            payload.kind === "transcript.final") &&
          (turn.phase === "listening" || turn.phase === "thinking")
        ) {
          setTranscript(typeof payload.text === "string" ? payload.text : "")
        }
        return
      }
      if (
        event.type === "permission.requested" &&
        payload.source === "voice" &&
        turn.phase === "thinking"
      ) {
        void invoke("focus_main_window", {
          conversationId: turn.conversationId,
        })
          .then(() => collapse(turn.generation))
          .catch((reason) => failCapture(turn.generation, reason))
        return
      }
      if (
        event.type === "model.started" &&
        payload.source === "voice" &&
        turn.phase === "thinking" &&
        !turn.runId
      ) {
        turn.runId = event.runId ?? null
        return
      }
      // Only the response begun for this turn may update or dismiss the widget.
      if (!turn.runId || event.runId !== turn.runId) return
      if (event.type === "task.failed") {
        void cancelSpeech().then(() => collapse(turn.generation))
      } else if (
        payload.source === "voice" &&
        event.type === "model.delta" &&
        (turn.phase === "thinking" || turn.phase === "responding")
      ) {
        const delta = typeof payload.delta === "string" ? payload.delta : ""
        turn.phase = "responding"
        setPhase("responding")
        assistantStreamRef.current += delta
        setAssistantStream(assistantStreamRef.current)
        push(delta)
      } else if (
        payload.source === "voice" &&
        event.type === "model.completed" &&
        (turn.phase === "thinking" || turn.phase === "responding")
      ) {
        const text =
          typeof payload.text === "string"
            ? payload.text
            : assistantStreamRef.current
        turn.phase = "speaking"
        setPhase("speaking")
        setAssistantStream(text)
        // Native speak resolves on the final audio callback, including all queued sentences.
        void finish(text).then((outcome) => {
          if (outcome === "finished") collapse(turn.generation)
        })
      }
    },
    [cancelSpeech, collapse, failCapture, finish, push]
  )

  const startCaptureRef = useRef(startCapture)
  const stopCaptureRef = useRef(stopCapture)
  const liveEventRef = useRef(handleLiveEvent)
  startCaptureRef.current = startCapture
  stopCaptureRef.current = stopCapture
  liveEventRef.current = handleLiveEvent

  useEffect(() => {
    let disposed = false
    const subscriptions: (() => void)[] = []
    let frame = 0
    let revision = 0
    const reveal = (visible: boolean) => {
      cancelAnimationFrame(frame)
      if (!visible) {
        if (turnRef.current) {
          void stopCaptureRef.current(true)
          void cancelSpeech()
        }
        setRevealed(false)
        clearWidget()
        return
      }
      frame = requestAnimationFrame(() => {
        frame = requestAnimationFrame(() => setRevealed(true))
      })
    }
    const subscribe = async <T,>(
      name: string,
      handler: (payload: T) => void
    ) => {
      const unlisten = await listen<T>(name, ({ payload }) => {
        if (!disposed) handler(payload)
      })
      if (disposed) unlisten()
      else subscriptions.push(unlisten)
    }
    void (async () => {
      await subscribe<boolean>("hud-reveal", (visible) => {
        revision++
        reveal(visible)
      })
      await subscribe<boolean>("hud-ptt", (pressed) => {
        if (pressed) void startCaptureRef.current()
        else void stopCaptureRef.current(false)
      })
      // The HUD consumes live events only; the dashboard owns conversation history.
      await subscribe<Event>("flowstate-event", (payload) => {
        try {
          liveEventRef.current(parseProtocolRecord("Event", payload))
        } catch {
          /* Reject unsupported protocol records. */
        }
      })
      if (disposed) return
      const before = revision
      const visible = await invoke<boolean>("hud_ready")
      if (!disposed && before === revision) reveal(visible)
    })().catch((reason) => {
      if (!disposed) setError(String(reason))
    })
    return () => {
      disposed = true
      subscriptions.forEach((unlisten) => unlisten())
      cancelAnimationFrame(frame)
      if (holdTimerRef.current !== null)
        window.clearTimeout(holdTimerRef.current)
    }
  }, [cancelSpeech, clearWidget])

  const beginHold = useCallback(() => {
    if (holdTimerRef.current !== null) return
    suppressClickRef.current = false
    holdTimerRef.current = window.setTimeout(() => {
      holdTimerRef.current = null
      void startCapture()
    }, 180)
  }, [startCapture])

  const endHold = useCallback(
    (cancel = false) => {
      if (holdTimerRef.current !== null) {
        window.clearTimeout(holdTimerRef.current)
        holdTimerRef.current = null
        return
      }
      void stopCapture(cancel)
    },
    [stopCapture]
  )

  const hideWidget = useCallback(() => {
    void stopCapture(true)
    void cancelSpeech()
    collapse(generationRef.current)
  }, [cancelSpeech, collapse, stopCapture])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault()
        hideWidget()
      }
    }
    window.addEventListener("keydown", onKeyDown)
    return () => window.removeEventListener("keydown", onKeyDown)
  }, [hideWidget])

  async function openDashboard() {
    await invoke("focus_main_window", {
      conversationId: conversationIdRef.current,
    })
  }

  const displayText = error || assistantStream || transcript
  const surface =
    expanded && displayText ? "expanded" : displayText ? "response" : "compact"
  const thinking = phase === "thinking"
  const active = phase !== "idle"
  useEffect(() => {
    void invoke("set_hud_surface", { surface }).catch(() => undefined)
  }, [surface])
  useEffect(() => {
    void invoke("set_hud_busy", { busy: active }).catch(() => undefined)
  }, [active])

  return (
    <main
      className="hud-stage"
      data-revealed={revealed}
      data-surface={surface}
      data-thinking={thinking}
      data-phase={phase}
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
            : phase === "listening"
              ? "Listening"
              : thinking
                ? "Thinking"
                : phase === "speaking"
                  ? "Speaking"
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
