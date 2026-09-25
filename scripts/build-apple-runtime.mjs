import { mkdir, copyFile } from "node:fs/promises";
import { arch } from "node:process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const packagePath = resolve(root, "native/apple-runtime");
const triple = arch === "arm64" ? "aarch64-apple-darwin" : "x86_64-apple-darwin";
const outputDir = resolve(root, "apps/desktop/src-tauri/binaries");
const output = resolve(outputDir, `flowstate-apple-runtime-${triple}`);

const proc = Bun.spawn(
  ["xcrun", "swift", "build", "-c", "release", "--package-path", packagePath],
  { cwd: root, stdout: "inherit", stderr: "inherit" },
);
const status = await proc.exited;
if (status !== 0) process.exit(status);

await mkdir(outputDir, { recursive: true });
await copyFile(resolve(packagePath, ".build/release/flowstate-apple-runtime"), output);
console.log(`Apple runtime: ${output}`);

