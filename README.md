# FlowState OS

Phase 0 scaffolds the three application shells and their shared protocol. The product requirements are in [FlowState-OS.md](FlowState-OS.md).

## Prerequisites

- Bun 1.3, Node.js 20.9 or newer, and Rust/Cargo
- macOS with Xcode or Xcode Command Line Tools for Tauri desktop development

## Setup and run

```sh
bun install
bun run generate:protocol
bun run check
```

Run each shell in its own terminal:

```sh
bun run dev:web       # http://localhost:3000
bun run dev:api       # http://localhost:8000/health
bun run dev:desktop   # Tauri window (Vite uses localhost:1420)
```

`bun run build:web` and `bun run build:desktop` build the web and desktop frontends. `cargo check --workspace` checks the Rust crate and Tauri host. From the root, Bun manages a single `bun.lock` and Cargo manages a single `Cargo.lock`.

## Ownership

| Path | Owner |
| --- | --- |
| `apps/web` | Next.js web shell |
| `apps/api` | Bun/Elysia cloud API shell |
| `apps/desktop` | Tauri/React desktop shell |
| `packages/protocol` | Canonical draft-07 JSON Schema, TypeScript types, validators, fixtures |
| `crates/flowstate-protocol` | Rust Serde types generated from the same schema at compile time |

The schema in `packages/protocol/schemas/protocol.schema.json` is the contract source of truth. Edit it, run `bun run generate:protocol`, update the fixtures, and run `bun run check`. The `Event` envelope has `schemaVersion: 1`; consumers must reject unsupported versions. This initial schema covers all 12 Phase 0 concepts and leaves detailed runtime behavior for later phases.
