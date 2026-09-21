import Ajv from "ajv";
import addFormats from "ajv-formats";
import schema from "../schemas/protocol.schema.json";
import type {
  Agent,
  Artifact,
  Device,
  Event,
  Memory,
  Model,
  Permission,
  Run,
  Skill,
  Task,
  Tool,
  Workflow,
} from "./generated";

export type { Agent, Artifact, Device, Event, Memory, Model, Permission, Run, Skill, Task, Tool, Workflow } from "./generated";

export const protocolVersion = 1 as const;
export const protocolKinds = [
  "Agent", "Task", "Run", "Tool", "Skill", "Workflow", "Artifact", "Event",
  "Model", "Permission", "Memory", "Device",
] as const;
export type ProtocolKind = (typeof protocolKinds)[number];

export type ProtocolRecords = {
  Agent: Agent;
  Task: Task;
  Run: Run;
  Tool: Tool;
  Skill: Skill;
  Workflow: Workflow;
  Artifact: Artifact;
  Event: Event;
  Model: Model;
  Permission: Permission;
  Memory: Memory;
  Device: Device;
};

const ajv = addFormats(new Ajv({ allErrors: true, strict: true }));
ajv.addSchema(schema);

const validators = Object.fromEntries(
  protocolKinds.map((kind) => [
    kind,
    ajv.getSchema(`${schema.$id}#/definitions/${kind}`),
  ]),
) as Record<ProtocolKind, ReturnType<typeof ajv.getSchema>>;

export function isProtocolRecord<K extends ProtocolKind>(kind: K, value: unknown): value is ProtocolRecords[K] {
  return validators[kind]?.(value) === true;
}

export function parseProtocolRecord<K extends ProtocolKind>(kind: K, value: unknown): ProtocolRecords[K] {
  const validate = validators[kind];
  if (!validate || !validate(value)) {
    throw new Error(`Invalid ${kind}: ${ajv.errorsText(validate?.errors)}`);
  }
  return value as ProtocolRecords[K];
}
