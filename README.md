# Tellur

**Know how AI participated in your codebase.**

Git records what changed. Tellur records how AI participated: which tool and
model were involved, which lines are attributable to an AI-assisted session,
what remains human or unknown, and whether the evidence is complete enough to
trust.

Tellur is local-first. Capture and inspection work without a cloud account. An
optional self-hosted Team Hub adds shared policy, repository access controls,
central reporting, SSO/SCIM, audit history, and organization-wide provenance.

> **Project status:** Tellur is pre-release software. The repository contains a
> verified v0.1 release pipeline and single-command installers. Marketplace
> listings are optional: the installer uses GitHub-hosted, checksum-verified
> VSIX and JetBrains ZIP release assets directly. See
> [PROJECT_STATUS.md](PROJECT_STATUS.md) for the dated implementation status and
> known blockers.

## The one setup flow

Install Tellur **once per machine**, from any directory. The installer downloads
a checksum-verified CLI, installs the editor package into detected VS Code,
Cursor, Windsurf, and JetBrains products, and immediately starts the setup
wizard. The wizard globally configures Codex, Claude Code, Gemini CLI,
Antigravity, Cursor, Windsurf, VS Code, and JetBrains plus an optional Team Hub.

macOS or Linux:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://github.com/sydneyvb-nl/tellur/releases/latest/download/install.sh | bash
```

Windows PowerShell:

```powershell
irm https://github.com/sydneyvb-nl/tellur/releases/latest/download/install.ps1 | iex
```

That is the normal setup path. The wizard is safe to run again and:

- configures supported agent hooks and MCP connections using the absolute path
  of the installed `tellur` binary;
- configures capture settings for VS Code-compatible editors;
- asks for a Team Hub URL, or keeps the installation local-only;
- performs machine-wide browser-based Team Hub device login when selected;
- remembers the most recently selected Team Hub as the default for unattended
  Git automation, even when credentials for multiple hubs are saved (`--local-only`
  explicitly disables implicit sync and removes the current repository's
  background service without deleting saved logins);
- makes every activated repository sync to that hub during `git push`;
- optionally installs interval-based background sync for the repository that is
  open while setup runs;
- activates every Git repository automatically on first configured
  agent/editor activity;
- initializes `.tellur/` and chained `post-commit`/`pre-push` automation during
  that first activity without overwriting existing shell hooks;
- configures `refs/notes/ai` automatically on the first Git push; and
- finishes with commands for checking and updating the setup.

There is no per-repository installation step. Opening or using a repository
through Codex, Claude Code, Gemini CLI, Antigravity, Cursor, Windsurf, VS Code,
or JetBrains is enough to activate it. Create `.tellur/disable` in a repository
that must opt out; the file stops agent/editor ingestion and makes Tellur's
managed commit/pre-push hooks and background service skip note publication and
Team Hub synchronization. An explicit manual `tellur push` also honors the opt-out.

For unattended installation, make the choice explicit:

```bash
tellur setup --local-only --yes
tellur setup --hub https://tellur.example.com --yes --no-browser
```

The wizard accepts HTTPS Team Hub URLs. Plain HTTP is accepted only for
`localhost` and `127.0.0.1` development hubs.

To update Tellur, rerun the same installer. It replaces the CLI, updates the
editor packages, and starts the idempotent wizard. If you only moved an existing
binary, reconcile every generated command, Git hook, and background service
without downloading anything:

```bash
tellur setup update
```

`tellur setup update` updates the **configuration to the currently running
binary**; the installer is the product-update mechanism.

Inspect the machine-wide setup and, when run inside one, the current repository:

```bash
tellur setup status
```

### What requires no development-time steps

Once setup is complete, supported agents submit events through their lifecycle
hooks, editor integrations capture saves, commits refresh Git AI notes, and Git
pushes flush local events to the configured Team Hub. Hub failures never block
`git commit` or `git push`; local provenance remains available for a later retry.

Prompt content is not stored by default. Tellur stores prompt hashes and
redacted metadata; a repository must explicitly enable redacted prompt excerpts.

### Editor package installation

Release assets contain one VSIX for VS Code-compatible editors and one JetBrains
plugin ZIP. The installer detects installed products and deploys those packages
before the wizard writes their settings. `tellur setup status` checks package
presence separately from configuration presence.

| Surface | Capture mechanism | Setup status |
| --- | --- | --- |
| Claude Code | Native lifecycle hooks | Configured automatically |
| Codex | Personal plugin hooks + MCP skill | Configured automatically |
| Gemini CLI | Native lifecycle hooks | Configured automatically |
| Antigravity | Native hooks + MCP | Configured automatically |
| Cursor | MCP + VS Code-compatible settings | Configured automatically |
| Windsurf | MCP + VS Code-compatible settings | Configured automatically |
| VS Code | Tellur extension save/watch capture | Release VSIX installed automatically when `code` is detected |
| JetBrains IDEs | Tellur VFS plugin | Release ZIP installed automatically into detected products |
| Aider, Copilot logs, Continue, Cline | Explicit import | Available, not live lifecycle capture |
| Devin | Authenticated daemon webhook | Available when a webhook is configured |

The VS Code extension and JetBrains plugin live in `editor/`. Marketplace
publication can improve discovery later, but is not required for the supported
installer path. The exact mechanics and limitations for every adapter are
documented in [docs/ADAPTERS.md](docs/ADAPTERS.md).

## What Tellur gives developers

### Line-level provenance

```bash
tellur explain src/auth.rs:42
tellur blame src/auth.rs
```

Tellur reports AI-assisted, human, and unknown attribution separately. It never
converts missing evidence into “human”. Attribution follows captured patches and
repository state; it is evidence, not authorship mind-reading.

### Session history and integrity

```bash
tellur sessions
tellur sessions <session-id>
tellur verify
```

Events are append-only JSONL with a SHA-256 hash chain and a local SQLite query
index. `verify` checks the recorded chain. Redaction is applied before sensitive
payload material is persisted.

### Pull-request risk reports

```bash
tellur pr-report --base main --head HEAD
```

The report combines changed-line attribution, tool/model evidence, policy
findings, and review coverage. A report with no matching attribution says that
the evidence is unknown or incomplete; it must not claim zero AI involvement.
For GitHub Actions, use the workflows in `.github/workflows/` as a starting
point and push `refs/notes/ai` when reports need commit-level provenance.

### Policy and portable evidence

```bash
tellur policy check
tellur export --format native --output provenance.json
tellur export --format agent-trace --output agent-trace.json
tellur notes show HEAD
```

Tellur supports repository policy, sensitive-path rules, native provenance
bundles, Agent Trace output, SLSA v1.0 provenance, SPDX 2.3 AI annotations, and
Git AI-compatible authorship notes.

`tellur policy check` evaluates every currently indexed attribution range
against `.tellur/policies/default.yml`. It exits non-zero when any policy
requirement fails and prints the affected rule, severity, file, and line range.
Use `tellur policy check --json` for a machine-readable CI result.

## Team Hub

The Team Hub is an optional self-hosted server for organizations that need a
shared control plane. The same `tellur setup` wizard connects a developer; there
is no separate normal onboarding flow.

The hub currently provides:

- SQLite for zero-config single-node use and Postgres for horizontal scale;
- organization and per-repository RBAC;
- API tokens plus OIDC SSO with browser/device login;
- SCIM 2.0 users and role-driving groups;
- tenant-scoped event and attribution ingestion;
- policy distribution, tamper-evident audit history, and durable export jobs;
- SLSA/SPDX export, dashboards, sessions, file provenance, and review gaps;
- private source browsing through an SSRF-guarded proxy; and
- GitHub App installation tokens and Git-note webhook harvesting.

Server deployment is intentionally separate from developer onboarding. See
[docs/proposals/TEAM_SERVER_IMPLEMENTATION.md](docs/proposals/TEAM_SERVER_IMPLEMENTATION.md)
and [`dist/docker/`](dist/docker/) for the current self-hosted deployment path.
The server crate has its own FSL-1.1-ALv2 license; the local core and CLI are
Apache-2.0.

## Data model and storage

Each initialized repository contains:

```text
.tellur/
├── config.yml
├── traces/          # append-only, hash-chained session events
├── index.db         # rebuildable SQLite query index
├── policies/        # repository or Team Hub policy
└── push_state.json  # per-hub incremental synchronization cursor
```

Generated files under `.tellur/` are ignored by the repository by default.
Commit-level provenance is published separately through `refs/notes/ai` when Git
automation is active. Anyone with repository read access may be able to read
those notes; remove the managed publication hooks with `tellur connect --remove`
if that is not appropriate for a repository.

## Supported capture guarantees

Tellur chooses the strongest integration a tool actually exposes:

1. native lifecycle hooks;
2. editor extension/plugin events;
3. MCP context and explicit recording tools;
4. authenticated daemon webhooks; or
5. import of an existing transcript/log.

These mechanisms are not equivalent. An MCP server cannot silently observe all
editor actions, an import cannot prove it saw events that were absent from the
source file, and editor save capture cannot identify an AI model unless the
editor provides that metadata. Tellur preserves those distinctions in its
source and confidence fields.

## Advanced and recovery commands

Most users should only need `tellur setup` and the inspection commands above.
The following remain available for CI, debugging, migration, and adapter work:

```bash
tellur doctor
tellur status
tellur setup agents
tellur connect --status
tellur login --hub https://tellur.example.com
tellur push --dry-run
tellur import <adapter> <path>
tellur daemon
tellur mcp
tellur redact
tellur gc --dry-run
```

`tellur watch` is a fallback for tools without a stronger integration. It scans
working-tree changes and therefore has weaker provenance than native hooks or
editor events; it is not part of the standard setup.

## Build and verify

Tellur pins its Rust toolchain. The repository-wide gates are:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test
cargo deny check
```

For source development, install the current CLI checkout with:

```bash
cargo install --path crates/cli --locked --force
tellur setup update
```

The dashboard, VS Code extension, and JetBrains plugin use their own toolchains.
Use the exact commands in [CONTRIBUTING.md](CONTRIBUTING.md); a passing Rust test
run does not verify TypeScript/Svelte/Kotlin code.

## Architecture

```text
Agent hooks / editor events / imports / webhooks
                         │
                         ▼
Rust adapters → redaction → hash-chained event log
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
      SQLite query index     line attribution
              │                     │
              └──────────┬──────────┘
                         ▼
 CLI / MCP / local UI / Git notes / optional Team Hub
```

- `crates/core/` — schema, storage, attribution, policy, reports, daemon, MCP
- `crates/cli/` — command-line UX, setup, imports, Git and Team Hub client
- `crates/adapters/` — source-specific normalization and sanitization
- `crates/server/` — optional Team Hub API, storage, auth, jobs and dashboard
- `editor/` — VS Code-compatible extension and JetBrains plugin
- `schemas/` — public JSON Schemas
- `web/` — local session replay dashboard

## Security and limitations

- Local daemon endpoints bind to loopback and require a token.
- Team Hub tenant endpoints deny access without an authenticated principal.
- Redaction reduces accidental secret storage but is not a substitute for
  secret scanning or repository access control.
- Hash chains reveal modification of recorded evidence; they do not prove that
  every real-world action was captured.
- Attribution quality is bounded by the integration and evidence available.
- Dashboard API values are rendered as text rather than injected as HTML; CI
  third-party actions are commit-pinned and release writes are job-scoped.

See [SECURITY.md](SECURITY.md), [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md), and
[docs/ADAPTERS.md](docs/ADAPTERS.md) before a production rollout. The latest
Aikido finding-by-finding triage is in
[docs/security/AIKIDO_2026_07_13.md](docs/security/AIKIDO_2026_07_13.md).

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) and [PROJECT_STATUS.md](PROJECT_STATUS.md)
before changing the code. The architecture map and repository working agreement
live in [AGENTS.md](AGENTS.md).

## License

The local core, CLI, adapters, schemas, and editor integrations are licensed
under Apache-2.0. `crates/server/` is separately licensed under FSL-1.1-ALv2.
See [LICENSE](LICENSE) and [`crates/server/LICENSE`](crates/server/LICENSE).
