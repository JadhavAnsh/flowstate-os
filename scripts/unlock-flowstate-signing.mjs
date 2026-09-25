import { existsSync } from "node:fs";
import { homedir, platform } from "node:os";
import { join } from "node:path";

if (platform() !== "darwin") process.exit(0);

const keychainPath = join(
  homedir(),
  "Library",
  "Keychains",
  "flowstate-local-signing.keychain-db",
);

if (!existsSync(keychainPath)) {
  console.error(`FlowState signing keychain not found: ${keychainPath}`);
  process.exit(1);
}

const unlock = Bun.spawnSync([
  "security",
  "unlock-keychain",
  "-p",
  "flowstate-local-build",
  keychainPath,
]);

if (unlock.exitCode !== 0) {
  process.stderr.write(unlock.stderr);
  process.exit(unlock.exitCode);
}

console.log("FlowState signing keychain unlocked.");
