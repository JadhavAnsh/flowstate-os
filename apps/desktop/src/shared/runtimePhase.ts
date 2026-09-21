import type { Event } from "@flowstate/protocol";

export type RuntimePhase = "idle" | "listening" | "working";

export function phaseFromEvents(events: Event[]): RuntimePhase {
  let phase: RuntimePhase = "idle";
  for (const event of events) {
    if (event.type === "agent.started") {
      const payload = event.payload as { phase?: string };
      if (payload.phase === "listening") {
        phase = "listening";
      }
    }
    if (event.type === "agent.message") {
      const payload = event.payload as { kind?: string; partial?: boolean };
      if (payload.kind === "listening.stopped") {
        phase = "idle";
      }
    }
    if (event.type === "task.started" || event.type === "model.started" || event.type === "model.delta") {
      phase = "working";
    }
    if (event.type === "task.completed" || event.type === "task.failed" || event.type === "model.completed") {
      const payload = event.payload as { cancelled?: boolean };
      if (event.type === "task.failed" && payload.cancelled) {
        phase = "idle";
      } else if (event.type !== "task.failed") {
        phase = "idle";
      }
    }
  }
  return phase;
}

export function hudTitle(phase: RuntimePhase): { title: string; subtitle: string } {
  if (phase === "listening") {
    return { title: "◉ FlowState", subtitle: "Listening…" };
  }
  if (phase === "working") {
    return { title: "◉ FlowState", subtitle: "Working…" };
  }
  return { title: "◉ FlowState", subtitle: "What can I do?" };
}
