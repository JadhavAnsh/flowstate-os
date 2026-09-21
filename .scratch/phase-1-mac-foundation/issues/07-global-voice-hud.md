# 07: Global voice HUD

**What to build:** A global shortcut (e.g. ⌥ Space) opens a **HUD** overlay: idle **“◉ FlowState — What can I do?”**, **listening**, and **working** states driven by real **Events** from Core—not hardcoded mocks. User can open the full desktop app to see the same conversation, timeline, or task detail.

**Blocked by:** 02 — Local event bus (protocol Events → UI); 06 — Conversation + Task/Run loop (text)

**Status:** resolved

- [x] Global activation shortcut works while the app is running (macOS).
- [x] HUD reflects live runtime state (idle / listening / working) from the event bus where applicable.
- [x] HUD can hand off or link to the main window for the active conversation or run.
- [x] HUD is usable alongside the primary desktop window (not replacing the operating console).
