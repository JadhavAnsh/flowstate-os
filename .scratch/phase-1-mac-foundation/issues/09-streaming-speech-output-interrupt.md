# 09: Streaming speech output + interruption

**What to build:** Assistant replies are **spoken** (streaming TTS where the platform supports it). User **barge-in** stops playback promptly and returns to capture/listening so conversation feels continuous, per voice-first requirements.

**Blocked by:** 08 — Streaming speech input (STT → Core); 05 — Model abstraction with streaming (first BYOK provider)

**Status:** resolved

- [x] Spoken output plays for assistant messages initiated from voice (and ideally text, if wired uniformly).
- [x] User interrupt during playback stops TTS quickly and does not leave Core in a stuck state.
- [x] After interrupt, user can speak again without restarting the app.
- [x] Phase 1 goal met: user can speak naturally, get a model-backed reply through local runtime, with streaming voice in and out.
