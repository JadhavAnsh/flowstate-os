import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { parseProtocolRecord, type Event } from "@flowstate/protocol";

export function useFlowStateEvents(initial: Event[] = []) {
  const [events, setEvents] = useState<Event[]>(initial);

  useEffect(() => {
    const unlisten = listen<Event>("flowstate-event", (payload) => {
      try {
        const event = parseProtocolRecord("Event", payload.payload);
        setEvents((prev) => [...prev, event]);
      } catch {
        // reject unsupported schema versions
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return { events, setEvents };
}
