# FlowState OS

## Product Requirements Document

**Document status:** Draft v1.0
**Product:** FlowState OS
**Primary launch platform:** macOS
**Future platforms:** iOS, Android, Windows, Linux, Web
**Product category:** Local-first AI operating layer / autonomous personal computing platform

---

# 1. Executive Summary

FlowState OS is a **voice-first, local-first AI operating layer** designed to let users control their computer, write software, execute tasks, automate workflows, interact with applications, access personal context, and delegate ongoing work using natural language.

FlowState is not intended to be another AI chat client.

It is intended to become a persistent intelligence layer between the user and their digital environment.

The product combines five major capabilities:

1. **Voice-first interaction**
2. **Agentic coding**
3. **Computer and application control**
4. **Autonomous workflows and persistent agents**
5. **Local and cloud AI orchestration**

The core product experience is:

> Speak naturally to FlowState, tell it what outcome you want, and let it determine which models, applications, tools, files, APIs, agents, and workflows are required to accomplish the task.

Examples:

> “Flow, find out why authentication is failing, fix it, run the tests, and show me the diff.”

> “Every weekday morning, check Gmail, Calendar, GitHub, and Linear and tell me what actually needs my attention.”

> “Reply to this email saying Friday works, but ask whether we can move the meeting to 3.”

> “Look at this error and fix it.”

> “Open the latest proposal, send it to Sarah, and remind me if she hasn’t responded by Thursday.”

FlowState should attempt to accomplish these tasks directly rather than returning instructions explaining how the user could accomplish them.

The original FlowState concept already positioned the product as an environment where users delegate complete workflows and where the system can reason, write and execute code, invoke external APIs, manage state, and deliver results. This PRD expands that idea into a broader AI operating system architecture.

---

# 2. Product Vision

## 2.1 Vision

Create the personal AI operating layer that sits between a user and their computer.

FlowState should eventually understand:

* what the user is working on;
* what applications are currently open;
* what is visible on screen;
* what projects and repositories matter;
* what files and documents matter;
* which people the user interacts with;
* which services are connected;
* which workflows the user performs repeatedly;
* what actions FlowState is allowed to take;
* what information should remain local;
* which AI model is appropriate for a given task.

The user should primarily interact with that intelligence using voice.

---

# 3. Product Positioning

## 3.1 Core positioning

**FlowState OS is a voice-first, local-first AI operating layer that can understand your computer, write software, operate your applications, and autonomously execute work.**

## 3.2 Product mental model

FlowState has four visible capabilities:

### ASK

Talk to FlowState about anything in the user's working context.

### BUILD

Create and modify software, documents, reports, research, and other artifacts.

### ACT

Operate applications, files, websites, APIs, and system capabilities.

### AUTOMATE

Create persistent workflows and autonomous agents that continue executing work over time.

All four capabilities run on top of:

* FlowState Core
* FlowState Intelligence
* FlowState Voice
* FlowState Memory
* FlowState Skills
* FlowState Permission Engine

---

# 4. Product Principles

## 4.1 Voice first

Every major capability should be usable by voice whenever technically possible.

For every feature, product teams should ask:

* Can the user start this by voice?
* Can the user modify it by voice?
* Can the user interrupt it by voice?
* Can the user approve it by voice?
* Can the user ask about progress by voice?
* Can the user cancel it by voice?
* Can the user undo it by voice?

A feature that requires keyboard or mouse interaction for its normal workflow should be considered incomplete unless there is a technical or security reason for that requirement.

---

## 4.2 Local first

Whenever possible:

* data remains on the user's device;
* inference happens locally;
* commands execute locally;
* memory is stored locally;
* credentials remain local;
* computer context remains local.

Cloud infrastructure should primarily provide:

* account management;
* encrypted synchronization;
* OAuth relays;
* external webhooks;
* remote device communication;
* billing;
* optional cloud execution;
* collaboration.

---

## 4.3 Action over explanation

If FlowState safely knows how to perform a requested action, it should perform the action rather than merely describe the steps.

Bad:

> “To run your tests, open Terminal and type…”

Good:

> “Running the tests.”

---

## 4.4 Models are replaceable

FlowState should never depend conceptually on one model vendor.

The product owns:

* context;
* tools;
* memory;
* workflows;
* permissions;
* execution;
* user experience.

AI providers supply interchangeable intelligence.

---

## 4.5 Deterministic tools before visual automation

When several mechanisms can accomplish an action, FlowState should prefer:

1. Native system API
2. Application API
3. App Intent / deep integration
4. CLI
5. Browser DOM or accessibility interface
6. Accessibility automation
7. Visual computer-use automation

Computer vision and cursor automation should be a fallback rather than the primary mechanism.

---

## 4.6 Explicit user authority

Autonomy should always be bounded by user-defined permissions.

FlowState may become highly autonomous, but the user remains the authority over:

* accessible information;
* connected services;
* external communications;
* financial actions;
* destructive actions;
* code execution;
* sensitive data;
* long-running agents.

---

# 5. Target Users

## 5.1 Primary personas

### Developer

Uses FlowState to:

* understand repositories;
* implement features;
* debug code;
* run applications;
* fix tests;
* review changes;
* operate Git;
* create pull requests;
* interact with deployment systems.

### Founder / operator

Uses FlowState to:

* triage email;
* prepare meetings;
* conduct research;
* monitor projects;
* automate administrative tasks;
* generate reports;
* coordinate tools;
* delegate repetitive operations.

### Power user

Uses FlowState as a general personal computing assistant capable of controlling applications and automating workflows.

### Researcher

Uses FlowState for:

* multi-source research;
* document synthesis;
* coding;
* data processing;
* recurring monitoring;
* artifact generation.

The original concept similarly targets developers, founders, researchers, and power users requiring more than a conversational chatbot.

---

# 6. Core Product Surfaces

The product should expose six primary surfaces while keeping voice as the universal entry point.

## 6.1 Ask

Conversational interface for questions, commands, and contextual assistance.

## 6.2 Code

Repository-aware coding environment.

## 6.3 Tasks

Finite agent jobs currently running or completed.

## 6.4 Agents

Persistent autonomous entities with goals, tools, permissions, schedules, and memory.

## 6.5 Automations

Trigger-based repeatable workflows.

## 6.6 Library

Artifacts, generated documents, outputs, reports, code changes, research, files, and reusable context.

---

# 7. Voice Experience

Voice is a foundational runtime, not a speech-to-text feature.

## 7.1 Entry mechanisms

Initial release:

* global keyboard shortcut;
* microphone button;
* desktop HUD.

Later:

* custom wake phrase;
* AirPods;
* mobile voice activation;
* Apple Watch;
* contextual widgets.

Example:

**⌥ Space**

HUD appears:

> What can I do?

User speaks immediately.

---

# 8. Voice Runtime

The Voice Runtime consists of:

* microphone capture;
* voice activity detection;
* wake-word or activation detection;
* speech recognition;
* interruption detection;
* conversational session state;
* intent routing;
* text-to-speech;
* audio output management.

Pipeline:

**Speech → Recognition → Intent → Context → Agent → Tools → Result → Voice response**

Speech should not simply become another chat message.

It should become structured user intent.

---

# 9. Continuous Conversation

FlowState must support conversational continuity.

Example:

User:

> “What meetings do I have tomorrow?”

FlowState:

> “Three. Product review at 11, Sam at 2, and investor call at 5.”

User:

> “Move Sam to Friday.”

FlowState:

> “Same time?”

User:

> “Yes.”

The Voice Session should retain:

* current subject;
* current task;
* referenced objects;
* selected application;
* active project;
* pending confirmation;
* recent entities;
* prior commands.

This enables:

* “send that”;
* “use the other one”;
* “same time”;
* “cancel it”;
* “fix that”;
* “commit it”;
* “do it again.”

---

# 10. Voice Interruptions

Users must be able to interrupt FlowState while it is speaking.

Example:

FlowState:

> “I found seventeen possible—”

User:

> “Only search the backend.”

Expected behavior:

1. FlowState stops speaking.
2. Active task pauses if necessary.
3. New instruction is applied.
4. Agent resumes with updated constraints.

This capability is mandatory for a natural conversational experience.

---

# 11. Voice Response Policy

FlowState should avoid unnecessarily verbose speech.

Three user-configurable response modes:

### Silent where possible

Perform obvious operations without spoken confirmation.

### Concise

Default.

Example:

> “Done. All tests pass.”

### Conversational

Provide more complete explanations.

Visual interfaces may still show complete details even when spoken output is concise.

---

# 12. Context Awareness

Voice becomes substantially more useful when combined with real-time device context.

FlowState may understand:

* active application;
* active window;
* selected text;
* cursor position;
* clipboard;
* current browser URL;
* accessibility hierarchy;
* focused UI element;
* open repository;
* open files;
* screen contents.

This allows commands such as:

> “Explain this.”

> “Reply to this.”

> “Fix this.”

> “Move this over there.”

> “Make this button smaller.”

> “Download that.”

Context resolution should prefer deterministic UI metadata over screenshot interpretation whenever available.

---

# 13. FlowState Core

The central product component is a local runtime called:

**FlowState Core**

FlowState Core should operate independently from the graphical UI.

Responsibilities:

* agent runtime;
* model routing;
* tool registry;
* skill registry;
* task scheduler;
* workflow execution;
* coding environment;
* computer control;
* memory;
* permissions;
* event processing;
* artifact handling;
* local persistence;
* audit trail.

The UI, CLI, mobile application, and web dashboard are clients of FlowState Core.

---

# 14. CLI

FlowState should expose major capabilities through a local command-line interface.

Examples:

`flow ask`

`flow run`

`flow agent`

`flow workflow`

`flow code`

`flow computer`

`flow skills`

`flow integrations`

The CLI should use the same runtime and event protocol as the desktop application.

---

# 15. Agent Runtime

Agents should be defined through configurable primitives rather than vendor-specific implementations.

Conceptual Agent object:

* identity;
* instructions;
* objective;
* available tools;
* model policy;
* memory policy;
* workspace;
* permission policy;
* execution status.

Basic loop:

**Intent → Context → Plan → Model → Tool → Permission → Execute → Observe → Continue / Ask / Finish**

The product may expose concise execution reasoning, progress, evidence, and rationale, but should not depend on displaying private model chain-of-thought.

---

# 16. Model Provider Layer

All models should implement a shared abstraction.

Required capabilities may include:

* text generation;
* streaming;
* tool calling;
* structured output;
* vision;
* embeddings;
* audio;
* long-context reasoning.

Candidate providers include:

* Apple on-device models;
* OpenAI;
* Anthropic;
* Google;
* OpenRouter;
* Ollama;
* LM Studio;
* MLX-backed models;
* custom OpenAI-compatible endpoints.

---

# 17. Model Router

Users should not need to manually choose a model for every command.

FlowState should select models based on:

* task complexity;
* privacy;
* latency;
* context size;
* modality;
* user policy;
* cost;
* connectivity;
* provider availability.

Suggested user modes:

* Auto
* Local Only
* Fastest
* Highest Capability
* Lowest Cost
* Custom

Example routing:

Simple application command → deterministic/local

Short summarization → local model

Intent classification → local model

Sensitive private document processing → local model

Repository-wide coding → coding model

Complex reasoning → high-capability model

Large research task → cloud model or delegated agents

---

# 18. Bring Your Own Key

BYOK is a first-class product requirement.

Users may connect:

* OpenAI;
* Anthropic;
* Gemini;
* OpenRouter;
* custom providers;
* local endpoints.

Users can configure:

* fast model;
* coding model;
* reasoning model;
* vision model;
* offline model;
* embedding model.

Provider credentials must not be stored unencrypted in ordinary application databases.

On macOS, credentials should be stored through secure system credential storage.

---

# 19. Apple-First Intelligence Strategy

macOS is the first target platform.

FlowState should integrate deeply with Apple's available local AI and native system capabilities wherever practical.

Architectural pattern:

**React interface → Tauri → Rust Core → Swift native bridge**

The Swift bridge can provide access to Apple-specific frameworks while Rust remains responsible for the cross-platform execution core.

FlowState must still retain its provider abstraction so Apple's intelligence layer is optional rather than structurally mandatory.

---

# 20. Desktop Technology

Recommended:

**Tauri 2 + React + TypeScript**

rather than Electron as the long-term architecture.

Reasons:

* smaller runtime footprint;
* Rust-native backend;
* stronger local systems integration;
* better separation of privileged operations;
* easier path toward secure desktop tooling.

Electron may remain a fallback if development constraints require it, but it should not be the preferred architecture.

---

# 21. Coding Harness

Coding is a first-class FlowState capability.

Required systems:

* repository manager;
* project detection;
* code search;
* file editing;
* patch generation;
* terminal execution;
* pseudo-terminal support;
* build execution;
* test execution;
* diagnostics;
* Git operations;
* dependency management;
* browser verification;
* task isolation;
* artifact preview.

---

# 22. Coding Workspaces

Agent coding sessions should run in isolated workspaces.

Preferred approach:

Git worktrees when the repository supports Git.

Example:

`flowstate/task-124`

`flowstate/task-125`

Multiple coding agents can therefore operate concurrently without modifying the same working tree.

---

# 23. Coding Tools

Minimum agent tools:

* read file;
* write file;
* patch file;
* search files;
* grep;
* list directory;
* inspect repository;
* inspect Git status;
* inspect Git diff;
* execute shell command;
* run tests;
* read diagnostics;
* manage worktree;
* launch application;
* open browser;
* inspect web UI;
* create commit.

Later:

* GitHub PR creation;
* CI inspection;
* code review;
* deployment.

---

# 24. Coding Workflow Example

User:

> “Fix the onboarding bug and open a PR.”

FlowState should:

1. identify the repository;
2. inspect relevant code;
3. reproduce the problem;
4. create an isolated workspace;
5. implement the fix;
6. run tests;
7. run the application;
8. visually or structurally verify the result;
9. summarize changes;
10. request approval where required;
11. commit;
12. create a pull request.

The user's role should be to define intent and approve consequential actions, not micromanage every implementation step.

---

# 25. Computer Runtime

FlowState should eventually control the computer directly.

Required subsystems:

* WindowController
* AppController
* AccessibilityController
* ScreenUnderstanding
* MouseController
* KeyboardController
* ClipboardController
* BrowserController
* FileSystemController
* ShellController
* NotificationController

---

# 26. macOS Permissions

FlowState may require explicit access to:

* Accessibility;
* Screen Context;
* Automation;
* Microphone;
* Notifications;
* Files and Folders.

Permissions should be requested progressively rather than during a giant onboarding permission wall.

The application should explain why a capability is needed at the moment it becomes relevant.

---

# 27. FlowState Skills

Integrations should be standardized around a FlowState Skill model.

A Skill can define:

* available tools;
* authentication method;
* triggers;
* events;
* permissions;
* schemas;
* UI components;
* metadata.

Initial Skills:

* GitHub;
* Gmail;
* Google Calendar;
* Slack;
* Linear;
* Notion;
* Vercel;
* browser;
* filesystem;
* terminal.

Later:

* Figma;
* databases;
* cloud infrastructure;
* CRM tools;
* productivity systems;
* music;
* messaging;
* home automation.

The original concept already proposed integrations across GitHub, databases, communications, web automation, development infrastructure, and custom APIs.

---

# 28. MCP

FlowState should support MCP interoperability.

FlowState should function as:

* an MCP client;
* potentially an MCP server.

Internally, FlowState Skills should remain the richer product abstraction.

MCP should operate as an adapter into the wider ecosystem rather than becoming the internal architecture itself.

---

# 29. Custom API Integration

Users and developers should eventually be able to provide:

* OpenAPI specifications;
* MCP servers;
* custom local tools;
* custom skill definitions.

FlowState can automatically generate typed tool wrappers subject to permission review.

---

# 30. Workflows

Workflows are deterministic or semi-deterministic reusable automations.

A Workflow consists of:

* trigger;
* inputs;
* steps;
* conditions;
* agent steps;
* tools;
* retry policy;
* approval checkpoints;
* outputs.

Possible triggers:

* schedule;
* webhook;
* file change;
* email received;
* GitHub event;
* application event;
* user request;
* system event;
* agent event.

---

# 31. Voice Workflow Creation

Users should be able to create workflows through normal conversation.

Example:

> “Every weekday at eight, check my calendar, inbox, GitHub, and Linear. Only tell me if something actually needs my attention.”

FlowState should translate that into a structured workflow.

A visual editor may exist for inspection and advanced editing, but it should not be required for common automations.

---

# 32. Tasks vs Agents vs Workflows

FlowState should explicitly distinguish four concepts.

### Chat

Interactive conversation.

### Task

Finite objective with a beginning and end.

### Agent

Persistent autonomous entity with a goal, tools, memory, and permissions.

### Workflow

Repeatable automation driven by structured triggers and steps.

These objects may interact but should remain independent product concepts.

---

# 33. Persistent Agents

Users should eventually create agents such as:

**Release Manager**

Goal:

Keep the product ready for release.

Capabilities:

* monitor GitHub;
* watch CI;
* inspect failing builds;
* create issues;
* generate release notes;
* prepare builds.

Restrictions:

* cannot merge without permission;
* cannot publish production releases without permission.

Agents may run on schedules or respond to events.

---

# 34. Permission Engine

Permission control is a foundational requirement.

Every tool action should be assigned a risk category.

Suggested classes:

* READ
* WRITE
* EXECUTE
* EXTERNAL
* DESTRUCTIVE
* SENSITIVE
* FINANCIAL

Users may configure permission behavior by:

* tool;
* application;
* folder;
* project;
* integration;
* agent;
* workflow;
* action category.

---

# 35. Autonomy Levels

Suggested modes:

### Observe

Read only.

### Assist

Ask before state-changing actions.

### Trusted

Automatically execute user-approved categories.

### Autonomous

Execute within explicitly configured policies.

Autonomy should be configurable per agent and per tool.

---

# 36. Confirmation Examples

Safe:

> “Open Slack.”

No confirmation required.

Moderate:

> “Send this message to Alex.”

FlowState:

> “Send it now?”

Sensitive:

> “Delete these files.”

FlowState:

> “This will move 173 files to Trash. Continue?”

Very high risk actions may require both spoken and visual confirmation.

---

# 37. Memory

FlowState should have local-first persistent memory.

Memory types:

* working memory;
* session memory;
* project memory;
* user preference memory;
* procedural memory;
* artifact memory.

The original concept already includes persistent context, memory across sessions, background agents, and task queues as foundational capabilities.

---

# 38. Memory Retrieval

FlowState should retrieve relevant memory on demand.

It should not automatically insert the user's entire history into every prompt.

Retrieval may consider:

* task;
* project;
* people;
* active application;
* current artifact;
* recent activity;
* semantic similarity.

Users need controls for:

* inspecting remembered information;
* controlling project boundaries;
* disabling memory;
* excluding sensitive sources.

---

# 39. Local Data Architecture

Recommended local storage:

**SQLite + full-text search + local vector index + filesystem artifact store**

Possible tables:

* conversations;
* messages;
* tasks;
* runs;
* agents;
* workflows;
* tools;
* tool_calls;
* approvals;
* events;
* artifacts;
* memories;
* projects;
* provider configuration.

Sensitive credentials should remain in secure credential storage rather than SQLite.

---

# 40. Event Bus

FlowState Core should be event-driven.

Example event types:

* task.started;
* task.completed;
* agent.started;
* agent.message;
* model.started;
* model.completed;
* tool.started;
* tool.completed;
* permission.requested;
* permission.resolved;
* workflow.started;
* workflow.failed;
* artifact.created;
* file.changed.

The same event model should drive:

* desktop UI;
* CLI;
* mobile UI;
* notifications;
* audit history;
* remote synchronization.

---

# 41. Audit Trail

Every autonomous action should be inspectable.

Example:

10:32:01
User requested login fix.

10:32:04
Agent searched authentication code.

10:32:10
Agent modified auth.ts.

10:32:16
Tests executed.

10:32:21
1 test failed.

10:32:27
Agent modified session.ts.

10:32:34
Tests executed.

10:32:39
All 48 tests passed.

The original concept already identifies session replay and audit trails as desktop capabilities.

Auditability should be an execution primitive, not merely an optional interface.

---

# 42. Artifacts

Agent results should become persistent first-class artifacts.

Examples:

* code changes;
* reports;
* research;
* documents;
* spreadsheets;
* images;
* presentations;
* websites;
* diffs;
* plans;
* summaries.

A conversation may contain:

* messages;
* tasks;
* tool executions;
* artifacts;
* approvals;
* decisions.

Artifacts should support:

* versioning;
* opening;
* editing;
* sharing;
* exporting.

---

# 43. Browser Runtime

Browser automation should use deterministic browser tooling when available.

Capabilities:

* navigate;
* click;
* type;
* extract content;
* inspect DOM;
* inspect accessibility tree;
* upload;
* download;
* screenshot;
* run JavaScript;
* monitor network/activity where allowed.

Visual browser control should be used only when deterministic browser APIs cannot accomplish the action.

---

# 44. Execution Isolation

Code execution and external tools should operate through a sandbox and permission broker.

Execution modes may eventually include:

* host execution;
* restricted local sandbox;
* container;
* remote sandbox.

Untrusted repository code should not automatically receive unrestricted host access.

---

# 45. FlowState Desktop UX

The full desktop app should serve as an operating console rather than only a chat window.

Suggested home layout:

**Good afternoon.
What should we work on?**

Active:

* Fix onboarding bug — running tests
* Competitor research — three agents working

Automations:

* Morning brief — 8:00
* GitHub issue triage — continuous
* Weekly analytics — Monday

Devices:

* MacBook Pro — online
* iPhone — online

The primary UI can coexist with the global voice HUD.

---

# 46. HUD

The HUD is the fastest interaction surface.

Initial state:

**◉ FlowState**

> What can I do?

During execution:

**◉ Working…**

The HUD should show only the information required to understand or control the current task.

Users can expand into the full application for logs, artifacts, agent status, or configuration.

---

# 47. Mobile Product

Recommended framework:

**React Native**

The first mobile app should primarily act as a companion to FlowState running elsewhere.

Initial functionality:

* voice commands;
* chat;
* task status;
* push notifications;
* action approval;
* workflow triggering;
* agent monitoring;
* artifact access;
* remote Mac control.

Later, selected tasks may execute directly on-device.

---

# 48. Remote Agent Experience

Example:

User from iPhone:

> “What's happening with the production build?”

FlowState checks the connected development environment.

User:

> “Fix it.”

FlowState starts the coding agent on the Mac.

User:

> “Deploy it when tests pass.”

FlowState creates the appropriate conditional action.

This experience makes FlowState a persistent personal computing layer rather than a desktop-only utility.

---

# 49. Web Application

Technology:

**Next.js**

Responsibilities:

Public:
* marketing;
* product pages;
* pricing;
* documentation;
* downloads;
* changelog.

Authenticated:
* account;
* devices;
* integrations;
* subscriptions;
* usage;
* remote task overview;
* workflows;
* teams;
* synchronization settings.

The core computer agent should not run in the Next.js application.

---

# 50. Cloud Backend

Recommended:
**Bun + Elysia**

Primary responsibilities:
* authentication APIs;
* device registration;
* OAuth callbacks;
* integration metadata;
* cloud webhooks;
* encrypted synchronization;
* billing;
* account entitlements;
* team organization;
* remote notifications;
* remote agent relay;
* optional hosted execution.

The cloud is a control plane.

The user's device remains the preferred execution plane.

---

# 51. Cloud Data

Recommended:
* PostgreSQL;
* Redis-compatible queue/cache where necessary;
* object storage.

Cloud data may include:
* users;
* organizations;
* memberships;
* devices;
* integration metadata;
* subscriptions;
* workflow metadata;
* encrypted synchronization records.

Sensitive local content should not automatically be copied into cloud storage.

---

# 52. Authentication Architecture

Separate three security domains.

### FlowState identity

Examples:
* passkey;
* email authentication;
* Sign in with Apple;
* Google;
* GitHub.

### Connected services

Examples:
* GitHub OAuth;
* Gmail OAuth;
* Slack OAuth;
* Linear OAuth.

### AI provider credentials

Examples:
* OpenAI key;
* Anthropic key;
* Gemini key;
* OpenRouter key.

These systems should not be conflated.

---

# 53. Synchronization

Cloud synchronization should be optional and encrypted.

Potentially sync:
* settings;
* workflows;
* agent definitions;
* permissions;
* memory selected for sync;
* task metadata;
* artifacts selected by user.

Do not assume every user wants conversation history or sensitive project context stored remotely.

---

# 54. Shared Protocol

Desktop, CLI, mobile, and web clients should use a shared event protocol.

Common event representation should support:
* model streaming;
* tool execution;
* terminal output;
* task progress;
* agent messages;
* approval requests;
* workflow state;
* computer-use state;
* artifact generation.

This reduces platform-specific execution behavior.

---

# 55. Suggested Monorepo

`flowstate/`

`apps/web`
Next.js marketing and dashboard

`apps/api`
Bun + Elysia

`apps/desktop`
Tauri + React

`apps/mobile`
React Native

`apps/docs`

`crates/flowstate-core`

`crates/flowstate-runtime`

`crates/flowstate-computer`

`crates/flowstate-security`

`crates/flowstate-fs`

`crates/flowstate-shell`

`crates/flowstate-apple`

`packages/agents`

`packages/ai`

`packages/auth`

`packages/cli`

`packages/connectors`

`packages/events`

`packages/memory`

`packages/protocol`

`packages/sdk`

`packages/skills`

`packages/tools`

`packages/ui`

`packages/workflows`

`skills/github`

`skills/gmail`

`skills/calendar`

`skills/slack`

`skills/linear`

`skills/notion`

`skills/browser`

`skills/filesystem`

`skills/shell`

---

# 56. Language Strategy

Recommended:

### TypeScript

Use for:
* Next.js;
* React;
* React Native;
* integrations;
* agent schemas;
* product logic;
* Skill SDK;
* web services.

### Rust

Use for:
* privileged desktop runtime;
* filesystem;
* shell execution;
* computer runtime;
* security boundaries;
* task runtime;
* performance-sensitive local services.

### Swift

Use for:
* Apple-specific APIs;
* native AI frameworks;
* macOS/iOS integration;
* system capabilities requiring native Apple frameworks.

The purpose of this division is not language purity.

Each language should be used where it creates meaningful technical leverage.

---

# 57. Performance Requirements

Voice-first interaction makes latency especially important.

Target experience:

Activation to listening UI: effectively immediate.

Simple deterministic commands: perceived instant response.

Local intent recognition: sub-second target.

Voice interruption: immediate stop behavior.

UI streaming: continuous rather than waiting for full generation.

Long operations should acknowledge quickly:

> “Working on it.”

and continue asynchronously inside the local runtime.

---

# 58. Offline Requirements

FlowState should remain useful without internet access.

Offline functionality should include where possible:
* voice recognition;
* basic intent parsing;
* local AI;
* filesystem actions;
* application control;
* local coding;
* Git operations;
* memory;
* local workflows;
* system utilities.

Cloud-only integrations should degrade gracefully.

The UI should clearly distinguish:
* offline unavailable;
* authentication required;
* provider unavailable;
* permission blocked.

---

# 59. Reliability

Agent execution must be resumable.

Persistent runs should store:
* current state;
* completed steps;
* active tool;
* retries;
* outputs;
* approvals;
* failure details.

Application crashes should not silently destroy long-running tasks.

Workflows should support configurable retry behavior.

---

# 60. Security Requirements

Mandatory architectural principles:
* least privilege;
* explicit permissions;
* local credential vault;
* sandboxed execution where possible;
* auditable tool calls;
* clear external-action confirmations;
* project boundaries;
* sensitive action classification;
* encrypted cloud synchronization;
* revocable integrations.

Agents must never gain more authority than the user has granted them.

---

# 61. Privacy Requirements

FlowState should clearly communicate when information:
* stays local;
* goes to an AI provider;
* goes to an integration;
* is synchronized;
* is included in a remote task.

Users should be able to inspect model/provider activity for sensitive tasks.

A future privacy mode can enforce:

**Local Only**

where no task data leaves the device.

---

# 62. Initial MVP

The MVP should prove three core experiences.

## Experience 1 — Coding

User:

> “Clone this project, run it, find out why login is broken, fix it, test it, and show me what changed.”

Required capabilities:

* repository;
* shell;
* file editing;
* coding model;
* tests;
* Git diff;
* approvals.

## Experience 2 — Automation

User:

> “Every morning check Gmail, Calendar, GitHub, and Linear and tell me what needs my attention.”

Required capabilities:

* integrations;
* scheduler;
* workflows;
* agent reasoning;
* notifications.

## Experience 3 — Computer Context

User:

> “Look at this error and fix it.”

Required capabilities:

* screen/application context;
* active project detection;
* computer runtime;
* coding agent.

If these three workflows feel reliable and significantly faster than manual operation, the product thesis is validated.

---

# 63. MVP Feature Scope

Required:

* macOS Tauri application;
* FlowState Core;
* global voice HUD;
* push-to-talk / hotkey;
* speech recognition;
* spoken responses;
* voice interruptions;
* local SQLite storage;
* BYOK provider configuration;
* provider routing;
* repository detection;
* coding agent;
* terminal tool;
* filesystem tools;
* Git tools;
* test execution;
* browser automation;
* basic computer context;
* permissions;
* task timeline;
* artifacts;
* GitHub integration;
* Gmail integration;
* Calendar integration;
* workflow scheduler.

Not required for first MVP:

* marketplace;
* teams;
* Windows;
* Linux;
* Android;
* advanced multi-agent orchestration;
* wake word;
* full cloud execution;
* enterprise administration;
* hundreds of integrations.

---

# 64. Phase 0 — Core Protocol

Before extensive UI development, define schemas for:

* Agent;
* Task;
* Run;
* Tool;
* Skill;
* Workflow;
* Artifact;
* Event;
* Model;
* Permission;
* Memory;
* Device.

The protocol should be stable enough that desktop, mobile, web, and CLI can use the same concepts.

---

# 65. Phase 1 — Mac Foundation

Deliver:

* Tauri shell;
* React desktop application;
* Rust runtime;
* SQLite;
* voice HUD;
* streaming voice;
* model abstraction;
* BYOK;
* secure credential vault;
* local event bus;
* conversation/task persistence.

Goal:

A user can speak naturally to FlowState and interact with models through the local runtime.

---

# 66. Phase 2 — Coding Agent

Deliver:

* repositories;
* worktrees;
* filesystem;
* terminal;
* Git;
* search;
* patching;
* diagnostics;
* test runner;
* browser verification;
* diff viewer;
* approvals.

Goal:

FlowState can autonomously complete useful coding tasks.

---

# 67. Phase 3 — Computer Control

Deliver:

* active application context;
* selected text;
* clipboard;
* accessibility tree;
* screen understanding;
* mouse/keyboard fallback;
* window control;
* browser control.

Goal:

Commands such as “fix this,” “reply to this,” and “open that” work in context.

---

# 68. Phase 4 — Skills

Deliver initial first-party Skills:

* GitHub;
* Gmail;
* Calendar;
* Slack;
* Linear;
* Notion;
* Vercel.

Also deliver:

* Skill SDK;
* MCP client;
* OAuth framework;
* generic API integration.

Goal:

FlowState can act across the user's working ecosystem.

---

# 69. Phase 5 — Autonomy

Deliver:

* background task runner;
* workflows;
* triggers;
* schedules;
* persistent agents;
* retry handling;
* notifications;
* audit history;
* approval policies.

Goal:

FlowState continues useful work without constant user supervision.

---

# 70. Phase 6 — Cloud Control Plane

Deliver:

* Next.js website;
* FlowState account;
* Elysia API;
* billing;
* device registration;
* OAuth relay;
* encrypted synchronization;
* webhook receiver;
* remote task relay.

Goal:

Multiple devices and remote integrations can interact with FlowState securely.

---

# 71. Phase 7 — Mobile Companion

Deliver React Native application with:

* voice;
* task monitoring;
* approvals;
* agent controls;
* notifications;
* workflow triggering;
* artifact access;
* remote Mac commands.

Goal:

The user can communicate with FlowState from anywhere.

---

# 72. Phase 8 — Ambient Assistant

Deliver:

* wake phrase;
* advanced conversational sessions;
* AirPods integration where feasible;
* Apple Watch companion;
* improved context detection;
* proactive assistance;
* multimodal commands.

Goal:

FlowState begins behaving like an ambient personal computing assistant rather than a conventional application.

---

# 73. Phase 9 — Platform Ecosystem

Deliver:

* Skill marketplace;
* workflow templates;
* agent templates;
* developer SDK;
* team collaboration;
* hosted runners;
* enterprise policies;
* optional self-hosted infrastructure.

The original v0.1 roadmap similarly anticipated background agents, persistent memory, local models, plugins, CLI access, teams, and self-hosting.

---

# 74. Success Metrics

Initial product metrics should focus on useful work completed rather than chat engagement.

Primary metrics:

* successful task completion rate;
* autonomous completion rate;
* median voice-command latency;
* percentage of tasks completed without manual UI interaction;
* tool execution success rate;
* coding task verification success;
* workflow reliability;
* permission interruption rate;
* number of recurring workflows per active user;
* weekly tasks delegated;
* percentage of model calls completed locally;
* cost per completed task;
* user reversal/undo rate.

Long-term north-star metric:

**Meaningful tasks completed by FlowState per active user per week.**

---

# 75. Quality Metrics

A task should not be considered successful merely because an agent says it succeeded.

Whenever possible, success should be externally verified.

Examples:

Coding:

Tests pass.

Email:

Message exists in Sent.

Calendar:

Event reflects requested changes.

Deployment:

Deployment provider confirms success.

File action:

Expected file exists.

Browser task:

Resulting application state can be inspected.

Verification should be built into tool execution.

---

# 76. Voice Success Metrics

Track:

* transcription accuracy;
* activation latency;
* interruption response;
* intent resolution accuracy;
* follow-up reference resolution;
* command completion rate;
* unnecessary spoken response rate;
* average clarification requests;
* voice-to-action completion time.

The desired experience is that users can complete common tasks without looking at the UI.

---

# 77. Business Model

Potential long-term structure:

### Free

* local runtime;
* limited features;
* BYOK;
* local models;
* basic automations.

### Pro

* advanced workflows;
* remote access;
* encrypted sync;
* premium Skills;
* advanced agent features;
* hosted services where required.

### Teams

* shared agents;
* shared Skills;
* centralized policies;
* organizational memory;
* team workflow automation.

### Enterprise

* self-hosting;
* SSO;
* audit controls;
* security policies;
* managed model providers;
* compliance features.

BYOK should remain valuable even on paid plans.

FlowState revenue should come from the platform, not forcing users through proprietary model markup.

---

# 78. Competitive Differentiation

FlowState should not attempt to win because it has access to more AI models.

Model aggregation is easily replicated.

The defensible product layer is:

**Context + Tools + Memory + Permissions + Execution + Workflows + Voice**

FlowState becomes more valuable because it increasingly understands:

* the user's computer;
* the user's projects;
* the user's applications;
* the user's working patterns;
* the user's permissions;
* the user's workflows.

The model can change without destroying that relationship.

---

# 79. Core Product Moat

Conceptually:

**User Context Graph**

Computer

* Projects
* Files
* Applications
* Accounts
* Workflows
* Memory
* Preferences
* Agents
* Permissions

→ FlowState Core

→ Think + Act

The objective is not merely better generation.

It is better **situational understanding and execution**.

---

# 80. Key Product Risks

## Security

An agent capable of operating a computer has significant authority.

Mitigation:

* strict permission boundaries;
* action classification;
* sandboxes;
* confirmations;
* comprehensive audit trails.

## Reliability

Autonomous agents may misunderstand ambiguous requests.

Mitigation:

* deterministic tooling;
* contextual grounding;
* verification;
* undo support;
* clarification for consequential uncertainty.

## Voice latency

Slow voice interaction destroys the feeling of intelligence.

Mitigation:

* local recognition;
* local intent routing;
* streaming;
* fast deterministic commands.

## Model dependency

Provider APIs and capabilities change.

Mitigation:

* model abstraction;
* BYOK;
* multiple providers;
* local models.

## Platform restrictions

Operating-system policies can constrain automation.

Mitigation:

* native APIs;
* layered control strategies;
* progressive platform-specific implementations.

## Excessive scope

The full vision is significantly larger than a normal productivity application.

Mitigation:

Start with Mac + coding + voice + limited computer control + a few critical Skills.

---

# 81. Non-Goals for Initial Release

The first release should not attempt to:

* replace the entire operating system;
* support every application;
* build a full IDE;
* provide perfect general-purpose vision-based computer control;
* create hundreds of integrations;
* build its own frontier model;
* provide unrestricted autonomy;
* support every desktop platform simultaneously.

The MVP should prove that **voice + context + agents + tools** can materially improve real workflows.

---

# 82. Design Requirement

The visual design should communicate:

* speed;
* awareness;
* calmness;
* precision;
* progress;
* control.

Avoid making every action feel like a chat transcript.

Prefer:

* live states;
* task timelines;
* compact action cards;
* artifacts;
* progress surfaces;
* permission prompts;
* ambient HUD interactions.

Chat is one interface into FlowState, not the product architecture.

---

# 83. Product Language

Preferred language:

* “Working”
* “Done”
* “Needs approval”
* “Blocked”
* “Waiting”
* “Running tests”
* “Watching”
* “Scheduled”

Avoid unnecessary AI terminology in normal user flows.

Users care about the outcome more than whether an “agentic orchestration loop” executed it.

---

# 84. Product Name Hierarchy

Suggested architecture:

**FlowState OS**
Overall platform.

**FlowState Core**
Local execution runtime.

**FlowState Voice**
Voice interaction system.

**FlowState Intelligence**
Model routing and inference layer.

**FlowState Skills**
Integration ecosystem.

**FlowState Memory**
Context and memory system.

**FlowState Cloud**
Cloud control plane.

**FlowState Mobile**
Mobile companion.

This creates a coherent platform vocabulary without tying product concepts to individual model providers.

---

# 85. Final Product Definition

FlowState OS should ultimately behave like this:

The user does not think:

> “Which AI model should I use?”

or:

> “Which app should I open?”

or:

> “Which automation platform do I need?”

The user simply states an outcome.

> “Get the new release ready.”

FlowState understands the relevant project, checks GitHub, reviews CI, identifies failures, invokes a coding agent, runs tests, prepares release notes, requests approval for consequential actions, and reports back when the release is ready.

Or:

> “Handle everything important this morning.”

FlowState checks the systems the user has authorized, filters noise, executes safe routine actions, and asks the user only when judgment or approval is required.

That is the product.

**FlowState should not merely answer the user.**

**FlowState should help operate the user's digital world.**

And the primary way the user should control that world is simply by speaking.
