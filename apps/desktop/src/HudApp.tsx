import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseProtocolRecord, type Event } from "@flowstate/protocol";
import { ensureActiveConversation } from "./shared/conversation";
import { hudTitle, phaseFromEvents } from "./shared/runtimePhase";
import { useFlowStateEvents } from "./hooks/useFlowStateEvents";
import { useSpeechInput } from "./hooks/useSpeechInput";
import { useSpeechOutput } from "./hooks/useSpeechOutput";
import { useCommandDoubleTap } from "./hooks/useCommandDoubleTap";
import "./Hud.css";

export default function HudApp() {
  useCommandDoubleTap();
  const [conversationId, setConversationId] = useState<string | null>(null);
  const [partial, setPartial] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [assistantStream, setAssistantStream] = useState("");
  const pttActiveRef = useRef(false);
  const { events, setEvents } = useFlowStateEvents();
  const { cancel: cancelSpeech, reset: resetSpeech, speakDelta, speakAll } = useSpeechOutput();

  const phase = useMemo(() => phaseFromEvents(events), [events]);
  const copy = hudTitle(phase);

  useEffect(() => {
    ensureActiveConversation()
      .then(setConversationId)
      .catch((err) => setError(String(err)));
    invoke<Event[]>("list_events", { limit: 200 })
      .then((stored) => setEvents(stored.map((e) => parseProtocolRecord("Event", e))))
      .catch(() => undefined);
  }, [setEvents]);

  useEffect(() => {
    const last = events.length > 0 ? events[events.length - 1] : undefined;
    if (!last) return;
    if (last.type === "model.delta") {
      const payload = last.payload as { delta?: string };
      const delta = payload.delta ?? "";
      setAssistantStream((prev) => {
        const next = prev + delta;
        speakDelta(next);
        return next;
      });
    }
    if (last.type === "model.completed") {
      const payload = last.payload as { text?: string };
      if (payload.text) {
        speakAll(payload.text);
      }
      setAssistantStream("");
      resetSpeech();
    }
    if (last.type === "task.failed") {
      setAssistantStream("");
      resetSpeech();
    }
  }, [events, resetSpeech, speakAll, speakDelta]);

  const submitTranscript = useCallback(
    async (text: string) => {
      if (!conversationId || !text.trim()) return;
      setError(null);
      try {
        await invoke("send_message", {
          conversationId,
          content: text.trim(),
          source: "voice",
        });
      } catch (err) {
        setError(String(err));
      }
    },
    [conversationId],
  );

  const speech = useSpeechInput({
    onPartial: (text) => {
      setPartial(text);
      if (conversationId) {
        invoke("voice_transcript_partial", { conversationId, text }).catch(() => undefined);
      }
    },
    onFinal: (text) => {
      setPartial("");
      submitTranscript(text).catch(() => undefined);
    },
    onError: (message) => {
      pttActiveRef.current = false;
      setError(message);
    },
  });

  function onPushToTalkStart(event: React.PointerEvent<HTMLButtonElement>) {
    event.preventDefault();
    event.stopPropagation();
    if (!conversationId || pttActiveRef.current) return;

    pttActiveRef.current = true;
    setError(null);
    setPartial("");
    setAssistantStream("");
    cancelSpeech();
    resetSpeech();

    // Must run synchronously in the pointer gesture (await before start() crashes WebKit on macOS).
    speech.start();

    void (async () => {
      try {
        await invoke("cancel_active_run");
        await invoke("voice_listening_started", { conversationId });
      } catch (err) {
        pttActiveRef.current = false;
        speech.stop();
        setError(String(err));
      }
    })();
  }

  function onPushToTalkEnd(event: React.PointerEvent<HTMLButtonElement>) {
    event.preventDefault();
    event.stopPropagation();
    if (!conversationId || !pttActiveRef.current) return;

    pttActiveRef.current = false;
    speech.stop();
    void invoke("voice_listening_stopped", { conversationId }).catch(() => undefined);
  }

  async function openMain() {
    if (!conversationId) return;
    await invoke("focus_main_window", { conversationId });
  }

  return (
    <div className="hud-shell">
      <div className="hud-card">
        <div className="hud-copy">
          <strong>{copy.title}</strong>
          <span>{copy.subtitle}</span>
          {partial ? <p className="hud-partial">{partial}</p> : null}
          {assistantStream ? <p className="hud-stream">{assistantStream}</p> : null}
          {error ? <p className="hud-error">{error}</p> : null}
        </div>
        <div className="hud-actions">
          <button
            type="button"
            className="ptt"
            onPointerDown={onPushToTalkStart}
            onPointerUp={onPushToTalkEnd}
            onPointerCancel={onPushToTalkEnd}
          >
            Hold to talk
          </button>
          <button type="button" className="ghost" onClick={() => openMain()}>
            Open console
          </button>
        </div>
      </div>
    </div>
  );
}
