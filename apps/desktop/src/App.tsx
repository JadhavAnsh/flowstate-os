import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { getCurrentWindow } from "@tauri-apps/api/window"
import {
  parseProtocolRecord,
  protocolVersion,
  type Event,
} from "@flowstate/protocol"
import { ensureActiveConversation } from "./shared/conversation"
import { phaseFromEvents } from "./shared/runtimePhase"
import { useFlowStateEvents } from "./hooks/useFlowStateEvents"
import { useSpeechOutput } from "./hooks/useSpeechOutput"
import { useCommandDoubleTap } from "./hooks/useCommandDoubleTap"
import Onboarding from "./Onboarding"
import "./App.css"

type CoreHealth = { status: string; version: string; protocol_version: number }
type Message = {
  id: string
  conversation_id: string
  role: string
  content: string
  created_at: string
}
type Task = {
  id: string
  title: string
  status: string
  conversation_id: string | null
  created_at: string
  updated_at: string
}
type ProviderStatus = {
  provider_id: string
  connected: boolean
  default_model: string
}
type WidgetState = "ready" | "opening" | "unavailable"

const navItems = [
  "Dashboard",
  "Ask",
  "Tasks",
  "Agents",
  "Automations",
  "Library",
]

function statusLabel(status?: string) {
  if (!status) return "Waiting"
  if (status === "completed") return "Done"
  if (status === "failed") return "Blocked"
  if (status === "running") return "Working"
  return status.charAt(0).toUpperCase() + status.slice(1)
}

function DashboardApp({
  onRestartOnboarding,
}: {
  onRestartOnboarding: () => void
}) {
  useCommandDoubleTap()
  const [health, setHealth] = useState<CoreHealth | null>(null)
  const [conversationId, setConversationId] = useState<string | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [tasks, setTasks] = useState<Task[]>([])
  const [draft, setDraft] = useState("")
  const [streaming, setStreaming] = useState("")
  const [voicePartial, setVoicePartial] = useState("")
  const [provider, setProvider] = useState<ProviderStatus | null>(null)
  const [apiKeyDraft, setApiKeyDraft] = useState("")
  const [error, setError] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [widgetState, setWidgetState] = useState<WidgetState>("ready")
  const { speak, cancel: cancelSpeech } = useSpeechOutput(setError)
  const streamingRef = useRef("")
  const conversationIdRef = useRef(conversationId)
  conversationIdRef.current = conversationId

  const showWidget = useCallback(async () => {
    setWidgetState("opening")
    try {
      await invoke("show_hud")
      setWidgetState("ready")
    } catch {
      setWidgetState("unavailable")
    }
  }, [])

  useEffect(() => {
    if (window.location.pathname !== "/dashboard") {
      window.history.replaceState({ route: "dashboard" }, "", "/dashboard")
    }
    const mainWindow = getCurrentWindow()
    let wasMinimized = false
    let disposed = false
    const syncMinimizedState = async () => {
      try {
        const minimized = await mainWindow.isMinimized()
        if (!disposed && minimized && !wasMinimized) await showWidget()
        wasMinimized = minimized
      } catch {
        if (!disposed) setWidgetState("unavailable")
      }
    }
    const unlisten = mainWindow.onResized(() => void syncMinimizedState())
    const poll = window.setInterval(() => void syncMinimizedState(), 750)
    return () => {
      disposed = true
      window.clearInterval(poll)
      void unlisten.then((stop) => stop())
    }
  }, [showWidget])

  const refreshMessages = useCallback(async (id: string) => {
    const rows = await invoke<Message[]>("list_messages", {
      conversationId: id,
    })
    setMessages(rows)
    setTasks(
      await invoke<Task[]>("list_tasks_for_conversation", {
        conversationId: id,
        limit: 10,
      })
    )
  }, [])

  const handleLiveEvent = useCallback(
    (event: Event) => {
      if (event.type === "agent.message") {
        const payload = event.payload as { partial?: boolean; text?: string }
        if (payload.partial && payload.text) setVoicePartial(payload.text)
      } else if (event.type === "model.delta") {
        streamingRef.current +=
          (event.payload as { delta?: string }).delta ?? ""
        setStreaming(streamingRef.current)
      } else if (
        event.type === "model.completed" ||
        event.type === "task.failed"
      ) {
        const payload = event.payload as { text?: string; source?: string }
        if (
          event.type === "model.completed" &&
          payload.text &&
          payload.source !== "voice"
        )
          speak(payload.text)
        streamingRef.current = ""
        setStreaming("")
        setVoicePartial("")
        if (conversationIdRef.current)
          refreshMessages(conversationIdRef.current).catch(() => undefined)
      }
    },
    [refreshMessages, speak]
  )
  const { events, setEvents } = useFlowStateEvents(handleLiveEvent)
  const runtimePhase = useMemo(() => phaseFromEvents(events), [events])

  const bootstrap = useCallback(async () => {
    setError(null)
    const [h, stored, status] = await Promise.all([
      invoke<CoreHealth>("core_health"),
      invoke<Event[]>("list_events", { limit: 200 }),
      invoke<ProviderStatus>("provider_status", { providerId: null }),
    ])
    setHealth(h)
    setEvents(stored.map((event) => parseProtocolRecord("Event", event)))
    setProvider(status)
    const active = await ensureActiveConversation()
    setConversationId(active)
    await refreshMessages(active)
  }, [refreshMessages, setEvents])

  useEffect(() => {
    bootstrap().catch((err) => setError(String(err)))
  }, [bootstrap])

  const timeline = useMemo(
    () =>
      [...events]
        .reverse()
        .slice(0, 5)
        .map((event) => ({
          id: event.id,
          type: event.type,
          time: new Date(event.occurredAt).toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
          }),
        })),
    [events]
  )

  async function onSaveKey() {
    setError(null)
    setProvider(
      await invoke<ProviderStatus>("set_provider_api_key", {
        providerId: null,
        apiKey: apiKeyDraft,
      })
    )
    setApiKeyDraft("")
  }

  async function onSend() {
    if (!conversationId || !draft.trim()) return
    setBusy(true)
    setError(null)
    setStreaming("")
    streamingRef.current = ""
    cancelSpeech()
    try {
      await invoke("send_message", {
        conversationId,
        content: draft.trim(),
        source: "text",
      })
      setDraft("")
      await refreshMessages(conversationId)
    } catch (err) {
      setError(String(err))
    } finally {
      setBusy(false)
    }
  }

  const latestTask = tasks[0]
  const latestMessage = [...messages]
    .reverse()
    .find((message) => message.role === "assistant")

  return (
    <main className="dashboard-shell" data-route="dashboard">
      <aside className="sidebar" aria-label="Primary navigation">
        <div className="brand" aria-label="FlowState OS">
          <span className="brand-orb" aria-hidden="true" />
          <span>FlowState</span>
        </div>
        <nav>
          {navItems.map((item, index) => (
            <button
              key={item}
              type="button"
              className={index === 0 ? "nav-item active" : "nav-item"}
              aria-current={index === 0 ? "page" : undefined}
            >
              <span className="nav-glyph" aria-hidden="true">
                {index + 1}
              </span>
              {item}
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <button
            type="button"
            className="nav-item"
            onClick={onRestartOnboarding}
          >
            <span className="nav-glyph" aria-hidden="true">
              ⌘
            </span>
            Run setup again
          </button>
          <div className="device-state">
            <span className="status-dot" />
            <div>
              <strong>This Mac</strong>
              <span>Online</span>
            </div>
          </div>
        </div>
      </aside>

      <section className="dashboard">
        <header className="dashboard-header">
          <div>
            <p className="eyebrow">Dashboard</p>
            <h1>Good afternoon.</h1>
            <p>What would you like Flow to take care of?</p>
          </div>
          <button
            type="button"
            className="widget-button"
            onClick={() => void showWidget()}
          >
            <span className="widget-mini" aria-hidden="true">
              <i />
              <i />
              <i />
            </span>
            {widgetState === "opening" ? "Opening…" : "Show widget"}
          </button>
        </header>

        {widgetState === "unavailable" ? (
          <div className="notice" role="alert">
            <div>
              <strong>Widget didn’t open</strong>
              <span>Flow is still available here. Try the widget again.</span>
            </div>
            <button type="button" onClick={() => void showWidget()}>
              Try again
            </button>
          </div>
        ) : null}
        {error ? (
          <div className="notice error" role="alert">
            <strong>Flow needs attention</strong>
            <span>{error}</span>
          </div>
        ) : null}

        <section className="command-surface" aria-labelledby="command-title">
          <div className="command-orb" aria-hidden="true">
            <span />
          </div>
          <div className="command-copy">
            <p className="command-state">
              {runtimePhase === "idle" ? "Ready" : runtimePhase}
            </p>
            <h2 id="command-title">Ask Flow anything</h2>
            <p>
              Start a task, work with a file, or ask about what’s on your
              screen.
            </p>
          </div>
          <div className="command-input">
            <textarea
              value={draft}
              onChange={(event) => setDraft(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault()
                  void onSend()
                }
              }}
              placeholder="Tell Flow what you want done…"
              aria-label="Ask Flow"
              rows={2}
            />
            <button
              type="button"
              className="send-button"
              onClick={() => void onSend()}
              disabled={busy || !draft.trim()}
              aria-label="Send"
            >
              {busy ? "···" : "↑"}
            </button>
          </div>
          <div className="command-hints">
            <span>
              <kbd>control</kbd> + <kbd>option</kbd> widget
            </span>
            <span>Hold the orb to talk</span>
          </div>
        </section>

        <div className="dashboard-grid">
          <section className="dashboard-card active-work">
            <div className="card-heading">
              <div>
                <p className="eyebrow">Current work</p>
                <h2>{latestTask?.title ?? "Nothing running"}</h2>
              </div>
              <span className={`state-pill ${runtimePhase}`}>
                {latestTask ? statusLabel(latestTask.status) : "Ready"}
              </span>
            </div>
            <div className="work-preview">
              <span className="work-icon">F</span>
              <div>
                <strong>
                  {voicePartial ||
                    streaming ||
                    latestMessage?.content ||
                    "Flow is ready for your next request."}
                </strong>
                <span>
                  {latestTask
                    ? `Updated ${new Date(latestTask.updated_at).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}`
                    : "Your work stays on this Mac."}
                </span>
              </div>
            </div>
          </section>
          <section className="dashboard-card activity-card">
            <div className="card-heading">
              <div>
                <p className="eyebrow">Activity</p>
                <h2>Recent events</h2>
              </div>
              <span className="count">{events.length}</span>
            </div>
            <ol className="activity-list">
              {timeline.length ? (
                timeline.map((item) => (
                  <li key={item.id}>
                    <span className="activity-mark" />
                    <strong>{item.type.replace(/\./g, " ")}</strong>
                    <time>{item.time}</time>
                  </li>
                ))
              ) : (
                <li className="empty">No activity yet</li>
              )}
            </ol>
          </section>
          <section className="dashboard-card connection-card">
            <div className="card-heading">
              <div>
                <p className="eyebrow">Connection</p>
                <h2>Model provider</h2>
              </div>
              <span
                className={
                  provider?.connected ? "status-dot" : "status-dot offline"
                }
              />
            </div>
            <p>
              {provider?.connected
                ? `${provider.default_model} is ready.`
                : "Connect a provider to start completing requests."}
            </p>
            {!provider?.connected ? (
              <div className="key-row">
                <input
                  type="password"
                  value={apiKeyDraft}
                  onChange={(event) => setApiKeyDraft(event.target.value)}
                  placeholder="OpenAI API key"
                  aria-label="OpenAI API key"
                />
                <button
                  type="button"
                  onClick={() => void onSaveKey()}
                  disabled={!apiKeyDraft}
                >
                  Connect
                </button>
              </div>
            ) : null}
            <div className="system-meta">
              Core {health?.status ?? "starting"} · v{health?.version ?? "…"} ·
              protocol v{health?.protocol_version ?? protocolVersion}
            </div>
          </section>
        </div>
      </section>
    </main>
  )
}

const ONBOARDING_KEY = "flowstate.onboarding.completed"

function App() {
  const [onboardingComplete, setOnboardingComplete] = useState(
    () => window.localStorage.getItem(ONBOARDING_KEY) === "true"
  )

  useEffect(() => {
    if (!onboardingComplete && window.location.pathname !== "/onboarding") {
      window.history.replaceState({ route: "onboarding" }, "", "/onboarding")
    }
  }, [onboardingComplete])

  if (!onboardingComplete) {
    return (
      <Onboarding
        onComplete={() => {
          window.localStorage.setItem(ONBOARDING_KEY, "true")
          setOnboardingComplete(true)
        }}
      />
    )
  }

  return (
    <DashboardApp
      onRestartOnboarding={() => {
        window.localStorage.removeItem(ONBOARDING_KEY)
        setOnboardingComplete(false)
      }}
    />
  )
}

export default App
