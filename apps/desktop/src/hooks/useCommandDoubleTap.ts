import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

const DOUBLE_TAP_MS = 450;

/** Double-tap ⌘ while a FlowState window is focused (no global keyboard hook). */
export function useCommandDoubleTap() {
  const lastTapRef = useRef(0);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Meta" && event.code !== "MetaLeft" && event.code !== "MetaRight") {
        return;
      }
      if (event.repeat) {
        return;
      }
      const now = Date.now();
      if (now - lastTapRef.current <= DOUBLE_TAP_MS) {
        event.preventDefault();
        lastTapRef.current = 0;
        void invoke("toggle_hud").catch(() => undefined);
        return;
      }
      lastTapRef.current = now;
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, []);
}
