# 05: Model abstraction with streaming (first BYOK provider)

**What to build:** User sends a text prompt from the desktop UI; Core routes it through a **model abstraction** to one BYOK-backed provider, **streams** tokens back, and emits **model.*** **Events** on the bus. A single default model/route is acceptable; full router modes are not required yet.

**Blocked by:** 04 — Secure credential vault + BYOK onboarding; 02 — Local event bus (protocol Events → UI)

**Status:** resolved

- [x] At least one provider works end-to-end using vault-stored credentials.
- [x] Streaming output is visible in the UI as it arrives (not only after completion).
- [x] Core emits appropriate **model.started** / progress / **model.completed** (or equivalent protocol events) during a call.
- [x] Errors from the provider surface to the user without crashing Core.
