# FlowState Desktop

Tauri/React desktop shell. See the [root README](../../README.md) for setup, commands, and protocol ownership.

## macOS voice HUD development

`bun run dev:desktop` runs the executable directly. On macOS, starting WebKit speech recognition from that process can cause a TCC crash claiming that `NSSpeechRecognitionUsageDescription` is missing, even though the key is in `src-tauri/Info.plist` and embedded in the executable. For voice testing, launch the app bundle instead:

```sh
bun --cwd apps/desktop tauri build --debug --bundles app
open 'target/debug/bundle/macos/FlowState OS.app'
```

The bundle contains the microphone and speech-recognition usage descriptions. Rebuild and relaunch after changes; this path does not provide the `tauri dev` hot-reload loop. Stop other running copies of FlowState OS before testing so the active window belongs to the newly built bundle. The system permission prompt still requires a user decision.
