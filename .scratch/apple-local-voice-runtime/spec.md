# Apple local voice runtime

Source: approved Apple-only local voice runtime plan.

**Goal:** A macOS 26 user can hold the existing HUD orb, speak in English (preferring `en-IN`), release, receive a streamed response from Apple Foundation Models, and hear it through native speech synthesis. Cloud inference is never invoked without an explicit approval.

**In scope:** SpeechAnalyzer/SpeechTranscriber, speech asset readiness, Foundation Models streaming and route assessment, AVSpeechSynthesizer, push-to-talk and barge-in, provider boundaries, permission-gated BYOK fallback, local diagnostics, packaged-app verification.

**Out of scope:** wake word, always-listening capture, speaker verification, stored audio, Whisper/GGUF, embeddings, reranking, local VLMs, non-macOS platforms.

**Constraints:** macOS 26 and Apple Silicon; protocol envelope remains version 1; no raw audio persistence; existing HUD structure remains intact; physical 8 GB and audible-output checks remain release gates.

