# 04: Secure credential vault + BYOK onboarding

**What to build:** User adds a model provider API key through desktop settings. Secrets live in **macOS secure credential storage**, not SQLite or plain files. UI shows connected/disconnected state; Core can resolve credentials when making outbound provider calls.

**Blocked by:** 01 — FlowState Core hosted in the desktop app

**Status:** resolved

- [x] User can save, update, and remove a BYOK provider credential through the UI.
- [x] Stored secrets use system secure storage; they never appear in logs, events, or SQLite.
- [x] Non-secret provider configuration (e.g. default endpoint, selected provider id) may live in local store separate from secrets.
- [x] Core exposes a clear failure when credentials are missing or invalid (user-visible, no silent fallback to anonymous access).
