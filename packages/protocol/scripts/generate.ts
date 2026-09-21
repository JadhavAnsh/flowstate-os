import { compile } from "json-schema-to-typescript";
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const source = fileURLToPath(new URL("../schemas/protocol.schema.json", import.meta.url));
const output = fileURLToPath(new URL("../src/generated.ts", import.meta.url));
const schema = JSON.parse(await readFile(source, "utf8"));
const generated = await compile(schema, "ProtocolRecord", {
  bannerComment: "// Generated from schemas/protocol.schema.json. Do not edit.\n",
  unreachableDefinitions: true,
});

if (process.argv.includes("--check")) {
  const current = await readFile(output, "utf8").catch(() => "");
  if (current !== generated) {
    throw new Error("Protocol TypeScript types are stale; run bun run generate:protocol");
  }
} else {
  await writeFile(output, generated);
}
