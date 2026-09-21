import { Elysia } from "elysia";
import { protocolVersion } from "@flowstate/protocol";

const app = new Elysia()
  .get("/", () => "FlowState API")
  .get("/health", () => ({ status: "ok", protocolVersion }))
  .listen(8000);

console.log(
  `FlowState API is running at ${app.server?.hostname}:${app.server?.port}`
);
