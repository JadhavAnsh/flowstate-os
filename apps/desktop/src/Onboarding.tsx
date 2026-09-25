import { useEffect, useMemo, useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { ensureActiveConversation } from "./shared/conversation"
import "./Onboarding.css"

type OnboardingProps = { onComplete: () => void }
type PermissionKey = "microphone" | "accessibility" | "screen"

const languages = ["English", "Marathi", "Hindi", "Spanish", "German"]
const appOptions = [
  ["Messages", "M", "green"],
  ["Mail", "M", "blue"],
  ["Reminders", "R", "orange"],
  ["Finder", "F", "cyan"],
  ["Notes", "N", "yellow"],
  ["Spotify", "S", "green"],
  ["Slack", "S", "violet"],
  ["Gmail", "G", "red"],
  ["Calendar", "31", "blue"],
  ["Notion", "N", "ink"],
] as const

function FlowMark({ large = false }: { large?: boolean }) {
  return (
    <span className={`flow-mark ${large ? "large" : ""}`} aria-hidden="true">
      <i />
      <i />
    </span>
  )
}

function ArrowIcon() {
  return (
    <svg viewBox="0 0 20 20" aria-hidden="true">
      <path d="M4 10h11M11 6l4 4-4 4" />
    </svg>
  )
}

function CheckIcon() {
  return (
    <svg viewBox="0 0 20 20" aria-hidden="true">
      <path d="m5 10 3 3 7-7" />
    </svg>
  )
}

export default function Onboarding({ onComplete }: OnboardingProps) {
  const [step, setStep] = useState(0)
  const [email, setEmail] = useState("")
  const [language, setLanguage] = useState("English")
  const [permissions, setPermissions] = useState<
    Record<PermissionKey, boolean>
  >({ microphone: false, accessibility: false, screen: false })
  const [pressedKeys, setPressedKeys] = useState({
    control: false,
    option: false,
  })
  const [shortcutVerified, setShortcutVerified] = useState(false)
  const [voiceHeld, setVoiceHeld] = useState(false)
  const [voiceTested, setVoiceTested] = useState(false)
  const [speechAssetProgress, setSpeechAssetProgress] = useState<number | null>(
    null
  )
  const [speechAssetError, setSpeechAssetError] = useState<string | null>(null)
  const [selectedApps, setSelectedApps] = useState<string[]>([
    "Finder",
    "Notes",
  ])
  const [calendar, setCalendar] = useState<string | null>(null)
  const [dictationTested, setDictationTested] = useState(false)
  const progress = useMemo(() => (step / 9) * 100, [step])

  useEffect(() => {
    if (step !== 4) return
    const update = (event: KeyboardEvent, active: boolean) => {
      if (event.key === "Control")
        setPressedKeys((current) => ({ ...current, control: active }))
      if (event.key === "Alt")
        setPressedKeys((current) => ({ ...current, option: active }))
      if (active && event.ctrlKey && event.altKey) setShortcutVerified(true)
    }
    const down = (event: KeyboardEvent) => update(event, true)
    const up = (event: KeyboardEvent) => update(event, false)
    window.addEventListener("keydown", down)
    window.addEventListener("keyup", up)
    return () => {
      window.removeEventListener("keydown", down)
      window.removeEventListener("keyup", up)
    }
  }, [step])

  useEffect(() => {
    if (step !== 5) return
    let disposed = false
    let unlisten: (() => void) | undefined
    void (async () => {
      unlisten = await listen<{
        event: string
        payload: { fraction?: number }
      }>("apple-runtime-event", ({ payload }) => {
        if (!disposed && payload.event === "asset_progress")
          setSpeechAssetProgress(payload.payload.fraction ?? 0)
      })
      setSpeechAssetProgress(0)
      await invoke("install_speech_assets")
      if (!disposed) setSpeechAssetProgress(1)
    })().catch((reason) => {
      if (!disposed) setSpeechAssetError(String(reason))
    })
    return () => {
      disposed = true
      unlisten?.()
    }
  }, [step])

  const next = () =>
    step === 9 ? onComplete() : setStep((current) => Math.min(9, current + 1))
  const back = () => setStep((current) => Math.max(0, current - 1))
  const togglePermission = (key: PermissionKey) =>
    setPermissions((current) => ({ ...current, [key]: !current[key] }))
  const toggleApp = (name: string) =>
    setSelectedApps((current) =>
      current.includes(name)
        ? current.filter((item) => item !== name)
        : [...current, name]
    )

  return (
    <main className="onboarding-shell" data-route="onboarding">
      {step > 0 ? (
        <div
          className="onboarding-progress"
          aria-label={`Setup step ${step + 1} of 10`}
        >
          <span style={{ width: `${progress}%` }} />
        </div>
      ) : null}
      <div className="onboarding-brand">
        <FlowMark />
        <span>FlowState OS</span>
      </div>
      <button type="button" className="skip-setup" onClick={onComplete}>
        Skip setup
      </button>

      <section className={`onboarding-stage step-${step}`}>
        {step === 0 ? (
          <div className="welcome-step">
            <div className="welcome-orb">
              <span className="orb-core" />
              <span className="orb-ring ring-one" />
              <span className="orb-ring ring-two" />
            </div>
            <FlowMark large />
            <h1>Meet Flow.</h1>
            <p>Your voice-first workspace for getting real work done.</p>
            <button
              type="button"
              className="primary-action hero-action"
              onClick={next}
            >
              Get started <ArrowIcon />
            </button>
            <span className="time-note">About 3 minutes</span>
          </div>
        ) : null}

        {step === 1 ? (
          <div className="split-step account-step">
            <div className="step-copy">
              <FlowMark />
              <h1>Welcome to FlowState</h1>
              <p>Move faster in every app with your voice.</p>
              <label className="text-field">
                <span>Email address</span>
                <input
                  type="email"
                  value={email}
                  onChange={(event) => setEmail(event.target.value)}
                  placeholder="you@example.com"
                />
              </label>
              <button
                type="button"
                className="primary-action wide"
                disabled={!email.includes("@")}
                onClick={next}
              >
                Continue with email
              </button>
              <div className="separator">
                <span>or continue with</span>
              </div>
              <button type="button" className="provider-button" onClick={next}>
                <span className="google-mark">G</span>Google
              </button>
              <button type="button" className="text-action" onClick={next}>
                Continue without an account
              </button>
            </div>
            <div className="account-visual" aria-hidden="true">
              <div className="blue-sphere" />
              <div className="mini-window">
                <span className="traffic-lights" />
                <i />
                <i />
                <i />
              </div>
              <div className="mini-widget">
                <b>F</b>
                <span />
                <span />
                <span />
              </div>
            </div>
          </div>
        ) : null}

        {step === 2 ? (
          <div className="center-step compact-step">
            <h1>What language do you speak?</h1>
            <p>Choose a primary language. You can add more later.</p>
            <div
              className="language-picker"
              role="radiogroup"
              aria-label="Primary language"
            >
              {languages.map((item) => (
                <button
                  key={item}
                  type="button"
                  role="radio"
                  aria-checked={language === item}
                  className={language === item ? "selected" : ""}
                  onClick={() => setLanguage(item)}
                >
                  <span className="radio-dot" />
                  {item}
                  {item === "English" ? <small>Recommended</small> : null}
                </button>
              ))}
            </div>
          </div>
        ) : null}

        {step === 3 ? (
          <div className="split-step permissions-step">
            <div className="step-copy permissions-copy">
              <h1>Enable core features</h1>
              <p>
                Flow asks only for access needed to complete work on this Mac.
              </p>
              <div className="permission-list">
                {(
                  [
                    [
                      "microphone",
                      "Microphone",
                      "Speak to Flow and transcribe your requests.",
                    ],
                    [
                      "accessibility",
                      "Accessibility",
                      "Let Flow insert text and control approved apps.",
                    ],
                    [
                      "screen",
                      "Screen recording",
                      "Let Flow understand what you choose to share.",
                    ],
                  ] as const
                ).map(([key, title, description]) => (
                  <button
                    key={key}
                    type="button"
                    className={
                      permissions[key]
                        ? "permission-row granted"
                        : "permission-row"
                    }
                    onClick={() => togglePermission(key)}
                  >
                    <span>
                      <strong>{title}</strong>
                      <small>{description}</small>
                    </span>
                    <i>{permissions[key] ? <CheckIcon /> : "Allow"}</i>
                  </button>
                ))}
              </div>
            </div>
            <div
              className="settings-preview"
              aria-label="System Settings preview"
            >
              <div className="settings-sidebar">
                <span />
                <span />
                <b />
                <span />
                <span />
              </div>
              <div className="settings-panel">
                <small>Privacy & Security</small>
                <h2>Allow FlowState OS</h2>
                {Object.entries(permissions).map(([key, granted]) => (
                  <div key={key}>
                    <i />
                    <span>
                      {key === "screen" ? "Screen & System Audio" : key}
                    </span>
                    <b className={granted ? "on" : ""} />
                  </div>
                ))}
              </div>
            </div>
          </div>
        ) : null}

        {step === 4 ? (
          <div className="center-step shortcut-step">
            <h1>Press these keys together</h1>
            <p>Use the shortcut anywhere to bring Flow into your work.</p>
            <div className="shortcut-card">
              <div className={`keycap ${pressedKeys.control ? "pressed" : ""}`}>
                <b>⌃</b>
                <span>control</span>
                <small>left</small>
              </div>
              <span className="plus">+</span>
              <div className={`keycap ${pressedKeys.option ? "pressed" : ""}`}>
                <b>⌥</b>
                <span>option</span>
                <small>left</small>
              </div>
            </div>
            <p
              className={
                shortcutVerified ? "verification success" : "verification"
              }
            >
              {shortcutVerified
                ? "Shortcut detected. You’re ready."
                : "Press and hold both keys to test it."}
            </p>
          </div>
        ) : null}

        {step === 5 ? (
          <div className="center-step voice-step">
            <h1>Ask a simple question</h1>
            <p>Try the interaction you’ll use every day.</p>
            <div className="voice-instruction">
              <span>1</span>
              <p>
                Hold <kbd>control</kbd> + <kbd>option</kbd>
              </p>
              <span>2</span>
              <p>Say “What can Flow help me with?”</p>
            </div>
            <button
              type="button"
              className={`hold-to-talk ${voiceHeld ? "active" : ""}`}
              disabled={speechAssetProgress !== 1}
              onPointerDown={() => {
                setVoiceHeld(true)
                void ensureActiveConversation()
                  .then((conversationId) =>
                    invoke("start_voice_capture", { conversationId })
                  )
                  .catch((reason) => setSpeechAssetError(String(reason)))
              }}
              onPointerUp={() => {
                setVoiceHeld(false)
                void invoke("stop_voice_capture")
                  .then(() => setVoiceTested(true))
                  .catch((reason) => setSpeechAssetError(String(reason)))
              }}
              onPointerCancel={() => {
                setVoiceHeld(false)
                void invoke("cancel_voice_capture")
              }}
            >
              <span className="mic-shape" />
              {voiceHeld
                ? "Listening…"
                : voiceTested
                  ? "Nice. Flow heard you."
                  : speechAssetProgress === 1
                    ? "Hold to talk"
                    : `Preparing speech… ${Math.round((speechAssetProgress ?? 0) * 100)}%`}
            </button>
            {speechAssetError ? (
              <p className="verification">{speechAssetError}</p>
            ) : null}
          </div>
        ) : null}

        {step === 6 ? (
          <div className="center-step apps-step">
            <h1>Choose where Flow can help</h1>
            <p>Pick a few apps now. Connections remain under your control.</p>
            <div className="app-grid">
              {appOptions.map(([name, mark, tone]) => (
                <button
                  key={name}
                  type="button"
                  className={selectedApps.includes(name) ? "selected" : ""}
                  onClick={() => toggleApp(name)}
                >
                  <i className={`app-icon ${tone}`}>{mark}</i>
                  <span>{name}</span>
                  {selectedApps.includes(name) ? (
                    <b>
                      <CheckIcon />
                    </b>
                  ) : null}
                </button>
              ))}
            </div>
          </div>
        ) : null}

        {step === 7 ? (
          <div className="center-step calendar-step">
            <h1>Connect your calendar</h1>
            <p>Flow can prepare you for meetings and protect focus time.</p>
            <div className="calendar-grid">
              {[
                ["Google Calendar", "G", "blue"],
                ["Outlook", "O", "cyan"],
                ["Apple Calendar", "17", "red"],
              ].map(([name, mark, tone]) => (
                <button
                  key={name}
                  type="button"
                  className={calendar === name ? "selected" : ""}
                  onClick={() => setCalendar(name)}
                >
                  <i className={`calendar-icon ${tone}`}>{mark}</i>
                  <strong>{name}</strong>
                  <small>
                    {calendar === name ? "Selected" : "Connect later"}
                  </small>
                </button>
              ))}
            </div>
          </div>
        ) : null}

        {step === 8 ? (
          <div className="split-step dictation-step">
            <div className="step-copy">
              <h1>Dictate in any text field</h1>
              <p>
                Hold your shortcut, speak naturally, and Flow writes where your
                cursor is.
              </p>
              <ul>
                <li>
                  <CheckIcon />
                  Works in the apps you already use
                </li>
                <li>
                  <CheckIcon />
                  Release the keys to stop
                </li>
                <li>
                  <CheckIcon />
                  You can edit before anything is sent
                </li>
              </ul>
            </div>
            <div className="dictation-demo">
              <div className="demo-toolbar">
                <span />
                <span />
                <span />
              </div>
              <p className={dictationTested ? "typed" : ""}>
                {dictationTested
                  ? "Schedule a focus block tomorrow at 10."
                  : "Click below to try a sample dictation."}
              </p>
              <button type="button" onClick={() => setDictationTested(true)}>
                {dictationTested ? "Sample complete" : "Try dictation"}
              </button>
              <div className="demo-lens">
                <i />
                <i />
                <i />
                <i />
              </div>
            </div>
          </div>
        ) : null}

        {step === 9 ? (
          <div className="center-step ready-step">
            <div className="ready-mark">
              <CheckIcon />
            </div>
            <h1>Flow is ready.</h1>
            <p>
              Start with one outcome. Flow will show what it’s doing and ask
              before anything sensitive.
            </p>
            <div className="ready-summary">
              <span>
                <b>{language}</b>Primary language
              </span>
              <span>
                <b>{Object.values(permissions).filter(Boolean).length} of 3</b>
                Permissions prepared
              </span>
              <span>
                <b>{selectedApps.length}</b>Apps selected
              </span>
            </div>
          </div>
        ) : null}
      </section>

      {step > 0 ? (
        <footer className="onboarding-footer">
          <button type="button" className="back-action" onClick={back}>
            Back
          </button>
          <span>{step + 1} of 10</span>
          <button type="button" className="primary-action" onClick={next}>
            {step === 9 ? "Open dashboard" : "Continue"}
            <ArrowIcon />
          </button>
        </footer>
      ) : null}
    </main>
  )
}
