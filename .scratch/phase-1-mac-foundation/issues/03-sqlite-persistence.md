# 03: SQLite persistence (conversations, messages, events, tasks)

**What to build:** Core persists **conversations**, **messages**, **tasks**, **runs**, and **events** in local SQLite. Provider secrets are not stored in SQLite. After quit and relaunch, conversation history and the event log reload in the main window.

**Blocked by:** 02 — Local event bus (protocol Events → UI)

**Status:** resolved

- [x] Local database opens on Core startup with migrations or equivalent schema management.
- [x] Events written to the bus are durably stored and can be replayed or listed after restart.
- [x] User can create or continue a **conversation** with **messages** that survive app restart.
- [x] **Task** and **Run** records can be created and retrieved aligned with protocol concepts (minimal fields acceptable for Phase 1).
- [x] Credentials and API keys are not stored in SQLite.
