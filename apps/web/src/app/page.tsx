import { protocolVersion } from "@flowstate/protocol";

export default function Home() {
  return (
    <main style={{ maxWidth: 720, margin: "5rem auto", padding: "0 1.5rem", fontFamily: "system-ui" }}>
      <h1>FlowState OS</h1>
      <p>Web shell ready.</p>
      <p>Shared protocol v{protocolVersion}</p>
    </main>
  );
}
