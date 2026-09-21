import { useCallback, useRef } from "react";

type SpeechRecognitionLike = {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  onresult: ((event: SpeechRecognitionEventLike) => void) | null;
  onerror: ((event: { error: string; message?: string }) => void) | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
  abort: () => void;
};

type SpeechRecognitionEventLike = {
  resultIndex: number;
  results: ArrayLike<{ isFinal: boolean; 0: { transcript: string } }>;
};

function getRecognitionCtor(): (new () => SpeechRecognitionLike) | null {
  const w = window as Window & {
    SpeechRecognition?: new () => SpeechRecognitionLike;
    webkitSpeechRecognition?: new () => SpeechRecognitionLike;
  };
  return w.SpeechRecognition ?? w.webkitSpeechRecognition ?? null;
}

export function useSpeechInput(options: {
  onPartial: (text: string) => void;
  onFinal: (text: string) => void;
  onError: (message: string) => void;
}) {
  const recognitionRef = useRef<SpeechRecognitionLike | null>(null);
  const activeRef = useRef(false);
  const onPartialRef = useRef(options.onPartial);
  const onFinalRef = useRef(options.onFinal);
  const onErrorRef = useRef(options.onError);
  onPartialRef.current = options.onPartial;
  onFinalRef.current = options.onFinal;
  onErrorRef.current = options.onError;

  const stop = useCallback(() => {
    activeRef.current = false;
    recognitionRef.current?.stop();
  }, []);

  const start = useCallback(() => {
    const Ctor = getRecognitionCtor();
    if (!Ctor) {
      onErrorRef.current(
        "Speech recognition is unavailable. Use the main window text Ask surface, or grant microphone and speech recognition access in System Settings.",
      );
      return;
    }

    if (activeRef.current) {
      return;
    }

    const recognition = new Ctor();
    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = "en-US";
    recognition.onresult = (event) => {
      let interim = "";
      let finalText = "";
      for (let i = event.resultIndex; i < event.results.length; i += 1) {
        const result = event.results[i]!;
        const transcript = result[0]?.transcript ?? "";
        if (result.isFinal) {
          finalText += transcript;
        } else {
          interim += transcript;
        }
      }
      if (interim.trim()) {
        onPartialRef.current(interim.trim());
      }
      if (finalText.trim()) {
        onFinalRef.current(finalText.trim());
      }
    };
    recognition.onerror = (event) => {
      activeRef.current = false;
      if (event.error === "not-allowed" || event.error === "service-not-allowed") {
        onErrorRef.current(
          "Microphone access was denied. Open System Settings → Privacy & Security → Microphone and Speech Recognition, enable FlowState OS, then try again.",
        );
        return;
      }
      onErrorRef.current(event.message ?? `Speech recognition error: ${event.error}`);
    };
    recognition.onend = () => {
      activeRef.current = false;
    };

    recognitionRef.current = recognition;
    activeRef.current = true;
    try {
      recognition.start();
    } catch (err) {
      activeRef.current = false;
      onErrorRef.current(String(err));
    }
  }, []);

  return { start, stop };
}
