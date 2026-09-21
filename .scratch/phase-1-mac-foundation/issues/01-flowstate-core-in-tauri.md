# 01: FlowState Core hosted in the desktop app

**What to build:** Launching the Mac desktop app starts **FlowState Core** inside the Tauri host—not a UI-only shell. The React client can invoke Core for health/version and treats Core as the single local execution entry point (same boundary future CLI/mobile clients will use).

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] Core initializes when the desktop app starts and shuts down cleanly on quit.
- [x] React can call Core and receive a stable health/version response (replacing or extending the current protocol-version stub).
- [x] Architecture makes Core the owner of runtime state; UI is a client, not the source of truth.
