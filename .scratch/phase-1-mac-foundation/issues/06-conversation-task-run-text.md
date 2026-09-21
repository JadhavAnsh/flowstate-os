# 06: Conversation + Task/Run loop (text)

**What to build:** Typed turns in an **Ask** surface create or update **Task** and **Run** records, persist **messages**, stream assistant replies via the model layer, and append **Events** for audit/history. Same persistence and task semantics voice will reuse later.

**Blocked by:** 03 — SQLite persistence (conversations, messages, events, tasks); 05 — Model abstraction with streaming (first BYOK provider)

**Status:** resolved

- [x] User can start or continue a conversation with streaming assistant replies (text input).
- [x] Each meaningful user request is associated with **Task** / **Run** state stored locally.
- [x] Messages and related **Events** persist and reload after restart (integrated with ticket 03).
- [x] UI distinguishes user vs assistant content and shows in-progress streaming state.
