import { useEffect, useRef, useState } from "react"
import { listen } from "@tauri-apps/api/event"
import { parseProtocolRecord, type Event } from "@flowstate/protocol"

export function useFlowStateEvents(onLiveEvent?: (event: Event) => void) {
  const [events, setEvents] = useState<Event[]>([])
  const onLiveEventRef = useRef(onLiveEvent)
  onLiveEventRef.current = onLiveEvent

  useEffect(() => {
    const unlisten = listen<Event>("flowstate-event", (payload) => {
      try {
        const event = parseProtocolRecord("Event", payload.payload)
        setEvents((prev) => [...prev, event])
        onLiveEventRef.current?.(event)
      } catch {
        // reject unsupported schema versions
      }
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  return { events, setEvents }
}
