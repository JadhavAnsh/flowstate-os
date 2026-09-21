import { useCallback, useRef } from "react";

export function useSpeechOutput() {
  const spokenRef = useRef("");
  const enabledRef = useRef(true);

  const cancel = useCallback(() => {
    if (typeof window !== "undefined" && "speechSynthesis" in window) {
      window.speechSynthesis.cancel();
    }
    spokenRef.current = "";
  }, []);

  const reset = useCallback(() => {
    spokenRef.current = "";
  }, []);

  const speakDelta = useCallback((fullText: string) => {
    if (!enabledRef.current || !fullText.startsWith(spokenRef.current)) {
      spokenRef.current = fullText;
      return;
    }
    const delta = fullText.slice(spokenRef.current.length);
    spokenRef.current = fullText;
    if (!delta.trim() || !("speechSynthesis" in window)) {
      return;
    }
    const utterance = new SpeechSynthesisUtterance(delta);
    utterance.rate = 1;
    window.speechSynthesis.speak(utterance);
  }, []);

  const speakAll = useCallback((text: string) => {
    if (!enabledRef.current || !text.trim() || !("speechSynthesis" in window)) {
      return;
    }
    spokenRef.current = text;
    const utterance = new SpeechSynthesisUtterance(text);
    window.speechSynthesis.speak(utterance);
  }, []);

  return { cancel, reset, speakDelta, speakAll, setEnabled: (v: boolean) => { enabledRef.current = v; } };
}
