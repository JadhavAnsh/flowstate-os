# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Four equal primary audiences, all requiring more than a conversational chatbot:

- **Developer** — understand repositories, implement features, debug, run apps, fix tests, review changes, operate Git, open PRs, talk to deployment systems.
- **Founder / operator** — triage email, prepare meetings, research, monitor projects, automate admin, generate reports, coordinate tools, delegate repetitive work.
- **Power user** — a general personal computing assistant that controls applications and automates workflows.
- **Researcher** — multi-source research, document synthesis, coding, data processing, recurring monitoring, artifact generation.

They speak to FlowState in the middle of real work on a computer (initially a Mac) and expect it to finish the job.

## Product Purpose

FlowState OS is a voice-first, local-first AI operating layer between the user and their digital environment. It is not a chat client.

Users tell it the outcome they want. It chooses models, applications, tools, files, APIs, agents, and workflows, then does the work.

Success is when three workflows feel reliable and faster than doing them by hand: autonomous coding (clone, diagnose, fix, test, show the diff), recurring automation (morning brief across mail, calendar, GitHub, Linear), and computer-context action (“look at this error and fix it”).

## Positioning

FlowState OS is a voice-first, local-first AI operating layer that can understand the user's computer, write software, operate applications, and autonomously execute work.

Visible capabilities:

- **ASK** — talk about anything in the working context
- **BUILD** — create and modify software, documents, reports, research, and other artifacts
- **ACT** — operate applications, files, websites, APIs, and system capabilities
- **AUTOMATE** — persistent workflows and agents that keep working over time

A neighboring chat product cannot truthfully copy this: FlowState owns context, tools, memory, permissions, execution, and the voice UX. Models are interchangeable intelligence underneath.

## Operating Context

Primary launch is macOS. The full desktop app is an operating console, not only a chat window, and it coexists with a global voice HUD. Web (Next.js) is marketing, account, devices, integrations, and remote overview — the core computer agent does not run there. Cloud (Bun/Elysia) is a control plane; the device is the preferred execution plane. Mobile is a later companion (voice, status, approvals, remote Mac control).

Users enter by global keyboard shortcut, microphone, or HUD. They address the product as **Flow**. Six primary surfaces sit under voice: Ask, Code, Tasks, Agents, Automations, Library.

This repo is Phase 0: three shells (`apps/desktop` Tauri/React, `apps/web` Next.js, `apps/api` Bun/Elysia) plus a shared protocol. Desktop is the primary design surface. Current UIs are scaffolds, not the product.

Confirmed terminology: Task (finite job), Agent (persistent entity with goals, tools, permissions, memory), Workflow (trigger-based repeatable automation), Artifact, Event (`schemaVersion: 1`; reject unsupported versions), Skill, Tool, Memory, Permission, Device, HUD.

## Capabilities and Constraints

Binding product behavior:

- Voice is a runtime, not speech-to-text bolted on. Every major capability must be startable, changeable, interruptible, approvable, cancellable, and undoable by voice unless security or technology forbids it.
- Local-first: data, inference, commands, memory, credentials, and computer context stay on-device when possible. Cloud is account, encrypted sync, OAuth relays, webhooks, remote devices, billing, optional cloud execution, collaboration.
- Action over explanation: if it can safely do the work, it does it (“Running the tests.”), not instructions for the user.
- Deterministic tools before visual automation: native API → app API → App Intent → CLI → DOM/a11y → accessibility automation → computer-use vision last.
- Explicit user authority: permissions bound autonomy over data, services, communications, money, destructive actions, code execution, and long-running agents.
- Models are replaceable. BYOK and routing are first-class.

MVP requires the macOS Tauri app, Core, global voice HUD, speech in/out with interruption, local SQLite, BYOK, coding agent + terminal/fs/Git/tests, basic computer context, permissions, task timeline, artifacts, GitHub/Gmail/Calendar, workflow scheduler.

Not in first MVP: marketplace, teams, Windows, Linux, Android, advanced multi-agent orchestration, wake word, full cloud execution, enterprise admin, hundreds of integrations.

Current codebase implements the Phase 0 protocol only. Detailed runtime behavior is later phases.

Undecided: accessibility standard (no WCAG target set), visual identity, and any non-PRD brand assets.

## Brand Commitments

- Product name: **FlowState OS**. Spoken address: **Flow**.
- HUD label: FlowState.
- Voice: do the work; do not lecture. Confirmations and progress are short and operational.
- Personality: persistent intelligence layer, not a chatbot personality pack.

No logo, type, or color system is binding. Do not invent a visual world from this file.

## Evidence on Hand

- Product requirements: `FlowState-OS.md` (Draft v1.0). Binding product truth; not visual direction.
- Protocol source of truth: `packages/protocol/schemas/protocol.schema.json`.
- Shells exist as placeholders (`apps/desktop`, `apps/web`, `apps/api`). They prove wiring, not UX.

Do not fabricate testimonials, customers, benchmarks, pricing, screenshots of a finished product, or third-party endorsements.

## Product Principles

1. **Voice first** — if it cannot be done by voice, the feature is incomplete unless security or platform blocks it.
2. **Local first** — the device is the execution and memory plane; cloud is control plane.
3. **Action over explanation** — accomplish the outcome; do not narrate how the user could.
4. **User remains the authority** — autonomy is earned through permissions, confirmations, and an audit trail.
5. **Own the layer, rent the model** — context, tools, memory, workflows, permissions, and execution are the product.
