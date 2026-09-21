import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseProtocolRecord, protocolVersion, type Event } from "@flowstate/protocol";
import { ensureActiveConversation } from "./shared/conversation";
import { phaseFromEvents } from "./shared/runtimePhase";
import { useFlowStateEvents } from "./hooks/useFlowStateEvents";
import { useSpeechOutput } from "./hooks/useSpeechOutput";
import { useCommandDoubleTap } from "./hooks/useCommandDoubleTap";
import "./App.css";

type CoreHealth = {
  status: string;
  version: string;
  protocol_version: number;
};

type Message = {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  created_at: string;
};

type Task = {
  id: string;
  title: string;
  status: string;
  conversation_id: string | null;
  created_at: string;
  updated_at: string;
};

type ProviderStatus = {
  provider_id: string;
  connected: boolean;
  default_model: string;
};

function App() {
  useCommandDoubleTap();
  const [health, setHealth] = useState<CoreHealth | null>(null);
  const { events, setEvents } = useFlowStateEvents();
  const [conversationId, setConversationId] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [draft, setDraft] = useState("");
  const [streaming, setStreaming] = useState("");
  const [voicePartial, setVoicePartial] = useState("");
  const [provider, setProvider] = useState<ProviderStatus | null>(null);
  const [apiKeyDraft, setApiKeyDraft] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const { speakDelta, speakAll, cancel: cancelSpeech, reset: resetSpeech } = useSpeechOutput();

  const runtimePhase = useMemo(() => phaseFromEvents(events), [events]);

  const refreshMessages = useCallback(async (id: string) => {
    const rows = await invoke<Message[]>("list_messages", { conversationId: id });
    setMessages(rows);
    const taskRows = await invoke<Task[]>("list_tasks_for_conversation", {
      conversationId: id,
      limit: 10,
    });
    setTasks(taskRows);
  }, []);

  const bootstrap = useCallback(async () => {
    setError(null);
    const h = await invoke<CoreHealth>("core_health");
    setHealth(h);
    const stored = await invoke<Event[]>("list_events", { limit: 200 });
    setEvents(stored.map((e) => parseProtocolRecord("Event", e)));
    const status = await invoke<ProviderStatus>("provider_status", { providerId: null });
    setProvider(status);
    const active = await ensureActiveConversation();
    setConversationId(active);
    await refreshMessages(active);
  }, [refreshMessages, setEvents]);

  useEffect(() => {
    bootstrap().catch((err) => setError(String(err)));
  }, [bootstrap]);

  useEffect(() => {
    const last = events.length > 0 ? events[events.length - 1] : undefined;
    if (!last) return;

    if (last.type === "agent.message") {
      const payload = last.payload as { partial?: boolean; text?: string };
      if (payload.partial && payload.text) {
        setVoicePartial(payload.text);
      }
    }

    if (last.type === "model.delta") {
      const delta = (last.payload as { delta?: string }).delta ?? "";
      setStreaming((prev) => {
        const next = prev + delta;
        speakDelta(next);
        return next;
      });
    }

    if (last.type === "model.completed" || last.type === "task.failed") {
      const completed = last.payload as { text?: string };
      if (last.type === "model.completed" && completed.text) {
        speakAll(completed.text);
      }
      setStreaming("");
      setVoicePartial("");
      resetSpeech();
      if (conversationId) {
        refreshMessages(conversationId).catch(() => undefined);
      }
    }
  }, [conversationId, events, refreshMessages, resetSpeech, speakAll, speakDelta]);

  const timeline = useMemo(
    () =>
      events.map((event) => ({
        id: event.id,
        label: `${event.type} · ${new Date(event.occurredAt).toLocaleTimeString()}`,
      })),
    [events],
  );

  async function onDevRun() {
    setError(null);
    const emitted = await invoke<Event[]>("trigger_dev_run");
    setEvents((prev) => [...prev, ...emitted.map((e) => parseProtocolRecord("Event", e))]);
  }

  async function onSaveKey() {
    setError(null);
    const status = await invoke<ProviderStatus>("set_provider_api_key", {
      providerId: null,
      apiKey: apiKeyDraft,
    });
    setProvider(status);
    setApiKeyDraft("");
  }

  async function onRemoveKey() {
    setError(null);
    const status = await invoke<ProviderStatus>("remove_provider_api_key", { providerId: null });
    setProvider(status);
  }

  async function onSend() {
    if (!conversationId || !draft.trim()) return;
    setBusy(true);
    setError(null);
    setStreaming("");
    cancelSpeech();
    resetSpeech();
    try {
      await invoke("send_message", {
        conversationId,
        content: draft.trim(),
        source: "text",
      });
      setDraft("");
      await refreshMessages(conversationId);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="container">
      <header className="header">
        <div>
          <h1>FlowState OS</h1>
          <p className="muted">
            Core {health?.status ?? "…"} · v{health?.version ?? "…"} · protocol v
            {health?.protocol_version ?? protocolVersion} · runtime {runtimePhase}
          </p>
        </div>
        <button type="button" className="secondary" onClick={() => onDevRun()}>
          Dev synthetic run
        </button>
      </header>

      {error ? <p className="error">{error}</p> : null}

      <section className="grid">
        <div className="panel">
          <h2>Ask</h2>
          {tasks[0] ? (
            <p className="muted task-meta">
              Latest task: {tasks[0].title} · {tasks[0].status}
            </p>
          ) : null}
          {voicePartial ? <p className="voice-partial">Voice (partial): {voicePartial}</p> : null}
          <div className="messages">
            {messages.map((m) => (
              <div key={m.id} className={`message ${m.role}`}>
                <strong>{m.role}</strong>
                <p>{m.content}</p>
              </div>
            ))}
            {streaming ? (
              <div className="message assistant streaming">
                <strong>assistant</strong>
                <p>{streaming}</p>
              </div>
            ) : null}
          </div>
          <div className="composer">
            <textarea
              value={draft}
              onChange={(e) => setDraft(e.target.value)}
              placeholder="Ask FlowState…"
              rows={3}
            />
            <button type="button" onClick={() => onSend()} disabled={busy}>
              {busy ? "Sending…" : "Send"}
            </button>
          </div>
        </div>

        <div className="panel">
          <h2>Event timeline</h2>
          <ul className="timeline">
            {timeline.map((item) => (
              <li key={item.id}>{item.label}</li>
            ))}
          </ul>
        </div>

        <div className="panel">
          <h2>BYOK · OpenAI</h2>
          <p className="muted">
            {provider?.connected ? "Connected" : "Not connected"} · model{" "}
            {provider?.default_model ?? "gpt-4o-mini"}
          </p>
          <p className="muted">HUD: double-tap ⌘ (while FlowState is focused) · hold to talk</p>
          <label className="field">
            API key
            <input
              type="password"
              value={apiKeyDraft}
              onChange={(e) => setApiKeyDraft(e.target.value)}
              placeholder="sk-…"
              autoComplete="off"
            />
          </label>
          <div className="row">
            <button type="button" onClick={() => onSaveKey()} disabled={!apiKeyDraft}>
              Save key
            </button>
            <button type="button" className="secondary" onClick={() => onRemoveKey()}>
              Remove key
            </button>
          </div>
        </div>
      </section>
    </main>
  );
}

export default App;
