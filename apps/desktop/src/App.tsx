import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { protocolVersion } from "@flowstate/protocol";
import "./App.css";

function App() {
  const [nativeVersion, setNativeVersion] = useState<number | null>(null);

  useEffect(() => {
    invoke<number>("protocol_version").then(setNativeVersion).catch(() => setNativeVersion(null));
  }, []);

  return (
    <main className="container">
      <h1>FlowState OS</h1>
      <p>Desktop shell ready.</p>
      <p>Shared protocol v{protocolVersion}</p>
      <p>Rust protocol v{nativeVersion ?? "connecting"}</p>
    </main>
  );
}

export default App;
