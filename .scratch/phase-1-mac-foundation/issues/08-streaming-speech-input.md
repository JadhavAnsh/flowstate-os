# 08: Streaming speech input (STT → Core)

**What to build:** Push-to-talk (or equivalent Phase 1 activation) in the **HUD**: microphone capture → **streaming speech recognition** → final transcript submitted as a user **message** on the same path as typed Ask (Task/Run, persistence, model routing).

**Blocked by:** 07 — Global voice HUD; 06 — Conversation + Task/Run loop (text)

**Status:** resolved

- [x] User can start and stop capture from the HUD; listening state is visible.
- [x] Partial/final transcript is shown in HUD (and/or main UI) during recognition.
- [x] Completed utterance triggers the same conversation/task flow as a typed message.
- [x] Microphone permission failures are explained to the user with a clear recovery path.
