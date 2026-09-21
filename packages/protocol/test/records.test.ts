import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { isProtocolRecord, parseProtocolRecord, protocolKinds } from "../src";

const fixtures = await Promise.all(
  protocolKinds.map(async (kind) => {
    const path = new URL(`../fixtures/${kind}.json`, import.meta.url);
    return [kind, JSON.parse(await readFile(path, "utf8"))] as const;
  }),
);

describe("canonical protocol fixtures", () => {
  for (const [kind, fixture] of fixtures) {
    test(`${kind} accepts its valid fixture`, () => {
      expect(isProtocolRecord(kind, fixture)).toBe(true);
      expect(parseProtocolRecord(kind, fixture)).toEqual(fixture);
    });

    test(`${kind} rejects a missing ID`, () => {
      const { id: _, ...invalid } = fixture;
      expect(isProtocolRecord(kind, invalid)).toBe(false);
    });
  }

  test("Event rejects an unsupported version", () => {
    const event = fixtures.find(([kind]) => kind === "Event")?.[1];
    expect(isProtocolRecord("Event", { ...event, schemaVersion: 2 })).toBe(false);
  });

  test("Event rejects unknown event types", () => {
    const event = fixtures.find(([kind]) => kind === "Event")?.[1];
    expect(isProtocolRecord("Event", { ...event, type: "unknown" })).toBe(false);
  });
});
