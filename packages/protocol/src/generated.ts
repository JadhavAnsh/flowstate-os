// Generated from schemas/protocol.schema.json. Do not edit.

/**
 * Phase 0 shared records. The Event envelope is versioned independently for transport compatibility.
 */
export type ProtocolRecord =
  Agent | Task | Run | Tool | Skill | Workflow | Artifact | Event | Model | Permission | Memory | Device;
export type RiskClass = "READ" | "WRITE" | "EXECUTE" | "EXTERNAL" | "DESTRUCTIVE" | "SENSITIVE" | "FINANCIAL";

export interface Agent {
  id: string;
  createdAt: string;
  updatedAt: string;
  name: string;
  instructions: string;
}
export interface Task {
  id: string;
  createdAt: string;
  updatedAt: string;
  title: string;
  status: "pending" | "running" | "completed" | "failed" | "canceled";
  agentId?: string;
}
export interface Run {
  id: string;
  createdAt: string;
  updatedAt: string;
  taskId: string;
  status: "pending" | "running" | "completed" | "failed" | "canceled";
}
export interface Tool {
  id: string;
  createdAt: string;
  updatedAt: string;
  name: string;
  riskClass: "READ" | "WRITE" | "EXECUTE" | "EXTERNAL" | "DESTRUCTIVE" | "SENSITIVE" | "FINANCIAL";
}
export interface Skill {
  id: string;
  createdAt: string;
  updatedAt: string;
  name: string;
  version: string;
}
export interface Workflow {
  id: string;
  createdAt: string;
  updatedAt: string;
  name: string;
  description?: string;
}
export interface Artifact {
  id: string;
  createdAt: string;
  updatedAt: string;
  runId: string;
  kind: string;
  uri: string;
}
export interface Event {
  id: string;
  schemaVersion: 1;
  type:
    | "task.started"
    | "task.completed"
    | "task.failed"
    | "agent.started"
    | "agent.message"
    | "model.started"
    | "model.delta"
    | "model.completed"
    | "tool.started"
    | "tool.output"
    | "tool.completed"
    | "permission.requested"
    | "permission.resolved"
    | "workflow.started"
    | "workflow.failed"
    | "artifact.created"
    | "terminal.output"
    | "computer.state"
    | "file.changed";
  occurredAt: string;
  runId?: string;
  payload: {
    [k: string]: unknown;
  };
}
export interface Model {
  id: string;
  createdAt: string;
  updatedAt: string;
  provider: string;
  name: string;
  capabilities: string[];
}
export interface Permission {
  id: string;
  createdAt: string;
  updatedAt: string;
  subjectId: string;
  toolId: string;
  decision: "allow" | "ask" | "deny";
  riskClass: RiskClass;
}
export interface Memory {
  id: string;
  createdAt: string;
  updatedAt: string;
  scope: "private" | "project" | "shared";
  content: string;
}
export interface Device {
  id: string;
  createdAt: string;
  updatedAt: string;
  name: string;
  platform: "macos" | "ios" | "windows" | "linux" | "android" | "web";
}
