import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./globals.css";
import App from "./App";
import HudApp from "./HudApp";

async function mount() {
  const label = getCurrentWindow().label;
  const Root = label === "hud" ? HudApp : App;
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <Root />
    </React.StrictMode>,
  );
}

mount();
