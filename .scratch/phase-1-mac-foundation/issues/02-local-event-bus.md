# 02: Local event bus (protocol Events → UI)

**What to build:** FlowState Core publishes **Events** on a local bus using the shared protocol envelope (`schemaVersion: 1`). The desktop UI subscribes and shows a live timeline (e.g. `task.started`, `model.started`, `model.completed`). Verifiable without real models via a dev trigger or synthetic run.

**Blocked by:** 01 — FlowState Core hosted in the desktop app

**Status:** resolved

- [x] Core emits protocol-valid **Events** that subscribers can receive in-process.
- [x] Desktop UI displays incoming events in real time (minimal timeline or log surface is enough).
- [x] Unsupported `schemaVersion` values are rejected per protocol rules.
- [x] A documented or built-in way proves the path end-to-end without BYOK or voice.
