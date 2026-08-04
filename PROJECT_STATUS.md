# Tellur — Project Status & Agent Guide

**Last updated:** 2026-08-04 (policy check enforcement)
**Maintained by:** agents — alle agents mogen dit updaten
**Repo:** github.com/sydneyvb-nl/tellur
**Branch:** `codex/document-v0-1-0-release`
**License:** Apache-2.0 (core) · FSL-1.1-ALv2 (`crates/server`)

## Handover — current state & open work

**PRs #48–#53 are merged to `main`: adapter/editor compatibility, team-hub UX,
evidence-aware provenance, unified setup/release packaging, and machine-wide
activation/release readiness, including the Aikido security remediation.** The
local pipeline, the team hub
(`tellur-server`), the CLI hub coupling (`tellur login`/`push`/`logout`), the A12
source connection + private-repo proxy, prompt excerpts, and the dynamic session
timeline are all shipped.

**Operational notes for whoever picks this up:**
- The hub embeds the SPA at build time, so **server or SPA changes need a
  `tellur-server` rebuild + restart** (or a fresh Docker image) to take effect;
  reinstall the CLI (`cargo install --path crates/cli --force`) after CLI changes.
- **Prompt excerpts** are opt-in per repo (`redaction.store_prompt_excerpt: true`
  in `.tellur/config.yml`) and apply only to activity captured *after* opting in
  (no backfill — older events are hash-only).
- An **`http` OIDC issuer on a LAN** (e.g. Keycloak on `192.168.x.x`) needs
  `TELLUR_OIDC_ALLOW_INSECURE_HTTP=1` on the hub (insecure; trusted networks only).
- The reference dev/test hub the user runs is on a LAN host; Postgres tests need
  `TELLUR_TEST_DATABASE_URL` (local: `postgres://postgres@127.0.0.1:5433/tellur_test`).

**Open work (priority order):**
1. **Zero-touch + GitHub App** — design merged in
   [`docs/proposals/GITHUB_APP.md`](docs/proposals/GITHUB_APP.md). **P1 `tellur
   connect`** is **complete** (git hooks + opt-in `--background` push service).
   **P2 GitHub App installation tokens** for the blob proxy is **complete** (see
   2026-06-13 below): the proxy mints short-lived per-repo `Contents:read`
   installation tokens for GitHub repos, with the PAT as the fallback. **P3 repo
   discovery + auto-provision + the `push`-webhook notes harvester** is complete:
   HMAC-verified inbound webhooks resolve a mapped installation id to one org,
   sync installed repos/source templates, fetch `refs/notes/ai` through the
   installation token, and ingest one commit-level event idempotently per
   `(repo, commit)`. Remaining GitHub-specific work: **P4** the PR risk-report
   **Check Run**. **Note:** P2/P3 are verified by unit + mock-client integration
   tests; a maintainer must verify them against a real GitHub App (App id +
   private key + webhook secret) — there is no live App in CI.
   Live setup walkthrough: [`docs/GITHUB_APP_SETUP.md`](docs/GITHUB_APP_SETUP.md).
2. **Additional package registries** — the direct GitHub installer is complete;
   the npm wrapper and Homebrew formula (`dist/`) remain unfinished optional
   distribution channels.
3. **`docs/DEPLOYMENT.md`** (Fly.io / Cloud Run + managed Postgres + TLS + OIDC
   walkthrough) and **`docs/SSO.md`** (Keycloak quickstart + the Entra ID
   `email_verified` caveat) — offered, not yet written; the missing piece for
   self-serve "host it as a service".
4. **Richer policy templates** for security-sensitive repositories.
5. **Broader agent coverage** as more tools expose stable local lifecycle hooks.

**Smaller follow-ups flagged during recent work (nice-to-have, not blocking):**
- Upgrade first-party GitHub Actions that still declare the deprecated Node 20
  action runtime. GitHub currently forces these actions onto Node 24, so this
  did not block `v0.1.0`, but the warnings should not remain indefinitely.
- Surface the **prompt excerpt as a column in the file-provenance view** (per
  range), not only in the session timeline.
- Enrich the **Sessions list** with per-session prompt/category indicators.

**Deliberate non-goal:** the hub's `review_coverage` / `reviewed_ai_lines` metric
has **no marking workflow on purpose** — review marking does not belong in the
hub (user decision). Leave it as a forward-looking metric; do not add a hub-side
"mark reviewed" action.

> **2026-08-04 — Policy-check enforcement (issue #56).** `tellur policy check`
> now evaluates every indexed attribution range against the default repository
> policy instead of only printing the YAML configuration. Failed policy results
> include severity, rule, file, and line range; any failure produces a non-zero
> exit status for CI. `--json` provides a machine-readable report. An end-to-end
> CLI test covers failing text/JSON output and the compliant success path.

> **2026-07-13 — Aikido security remediation.** On
> `codex/security-aikido-remediation`. Removed confirmed DOM XSS sinks from the
> embedded local dashboard by switching all API/import-driven rendering to DOM
> text nodes; added a regression test that rejects HTML string sinks. Replaced
> the SQLite hash-chain helper's dynamic table/column strings with a closed enum
> and static parameterized queries. Pinned every remaining reported third-party
> GitHub Action to a full commit SHA, retained PR #52's pinned cargo-deny CLI
> replacement, made release permissions deny-by-default and
> job-scoped, updated `smallvec` to 1.15.2, and upgraded `notify` to 8.2.0 so
> vulnerable `mio` 0.8.11 leaves the graph. The 15 path-read alerts were traced
> to explicit local-user imports or fixed RepoStorage/home paths, not a remote
> restricted-directory boundary; the non-applicability proof and security
> invariants are recorded in `docs/security/AIKIDO_2026_07_13.md`. Verified with
> `cargo fmt --all -- --check`, workspace clippy with warnings denied, all 359
> Rust tests, `cargo deny check`, JavaScript/YAML parsing, pinned-SHA resolution,
> and a real headless-browser injection attempt through the local daemon (the
> malicious `<img onerror>` remained literal text; zero injected elements and
> no handler execution).

> **2026-07-13 — v0.1.0 release workflow repair.** The first tag build exposed
> that the VS Code package was already version `0.1.0`; npm treats setting an
> identical version as an error by default and therefore skipped all editor
> artifacts and publication. The release workflow now explicitly allows the
> checked-in package version to match the tag. Production dependency audit is
> clean (`npm audit --omit=dev`: zero findings). The original `v0.1.0` run did
> not publish a GitHub Release; after this fix merges, the unpublished tag must
> be recreated on the repaired commit before rerunning the release pipeline.
> The VSIX packager was also upgraded to `@vscode/vsce` 3.9.2 with patched
> `form-data` 4.0.6 and `undici` 6.27.0; the full extension dependency audit now
> reports zero vulnerabilities.

> **2026-07-13 — v0.1.0 published.** Recreated the previously unpublished tag
> on PR #54's merge commit (`ed361398`) and completed the release workflow:
> editor packages, macOS arm64/x64, Linux arm64/x64, Windows x64, and GitHub
> Release publication all passed. The published release contains 16 assets:
> both installers, five platform archives, VSIX and JetBrains packages, and
> seven SHA-256 sidecars. Every checksum was independently verified after
> downloading the public assets; archive structure was inspected; stable
> `releases/latest/download` links returned HTTP 200 and matched the versioned
> installers. A clean temporary-home installation through the public Unix URL
> installed `tellur 0.1.0` and completed non-interactive global setup.

> **2026-07-13 — Unified setup and reality-aligned product onboarding.** On
> `codex/unified-setup-wizard`. `tellur setup` is now the supported idempotent
> entry point for machine-wide hook/MCP/settings generation, current-repository
> initialization, non-clobbering Git automation, optional secure Team Hub
> device login, and automatic background synchronization. `tellur setup update`
> reconciles generated absolute binary paths, Git hooks, and an existing push
> service after an upgrade; `tellur setup status` combines machine and repository
> health. The pre-push hook now configures Git-note fetch/rewrite support itself
> when a remote is added after onboarding, and notes config writes are
> idempotent. Rewrote the README around this single journey and removed false
> distribution claims. Follow-up packaging in the same PR adds a single stable
> install URL per operating-system family, checksum-verified binary downloads,
> version-matched VSIX and JetBrains ZIP assets, automatic installation into
> detected editors, package-aware setup status, a Unix installer E2E test, and
> a Windows installer E2E CI job.
> Marketplace listing is now optional for discovery rather than a blocker.
> Granular setup/connect commands remain for compatibility and recovery.

> **2026-07-13 — Machine-wide activation semantics.** On
> `codex/global-auto-activation`. Corrected the remaining per-repository wording
> and behavior before v0.1.0: Tellur installs once per machine; Codex, Claude
> Code, Gemini CLI, Antigravity, Cursor, Windsurf, VS Code, and JetBrains are
> explicitly reported by the wizard; every Git repo auto-initializes on first
> configured agent/editor activity. `tellur init` and hook `--auto-init` now
> install managed post-commit/pre-push hooks through one shared idempotent path,
> so portable notes and hub sync do not require a second repo setup command.
> Successful device login now records a validated machine-wide default hub, so
> unattended pre-push sync remains deterministic when multiple hub credentials
> exist. PR CI also cross-compiles and packages both Linux release targets;
> arm64 correctly uses the GNU cross toolchain instead of a GNU linker mislabeled
> as musl.
> Codex review P2 fixed: `.tellur/disable` is evaluated at hook runtime, so a
> repository that opts out after activation no longer exports/pushes Git notes
> or synchronizes events to the Team Hub. A shell-execution regression test
> covers disabled and re-enabled behavior.
> The supply-chain job now pins cargo-deny 0.19.8 instead of using an action
> that downloads an incompatible floating `latest` CLI.
> Codex re-review P1s fixed: `cmd_push` itself honors `.tellur/disable`, including
> timer services, and `--local-only` persists an explicit unattended-sync opt-out.
> Pre-push/service calls are marked unattended and never fall back to the sole
> saved host when no default hub is selected.
> Direct `tellur notes push` now propagates the pre-push recursion guard, avoiding
> concurrent nested pushes racing to update `refs/notes/ai`.
> Final Codex P2 fixed: local-only setup removes the current repository's existing
> background service, while the persisted opt-out also blocks implicit fallback
> for legacy timers that were created without the unattended marker.

> **2026-07-13 — End-to-end Codex capture and evidence-aware PR provenance.** On
> `fix/pr-provenance-evidence-states`. Replaced the broken CI use of local-index
> `tellur pr-report`: a fresh Actions checkout cannot contain the developer's
> local SQLite attribution and previously rendered that missing evidence as
> “Low risk / 0% AI / 0 total lines”. The workflow now fetches `refs/notes/ai`
> and runs the git-native team report against the immutable PR base/head SHAs.
> Team reporting parses each commit's zero-context patch, intersects portable
> authorship ranges with the actual added lines, reports deletions separately,
> and exposes `complete`/`partial`/`missing`/`empty` line-coverage states. Missing
> or invalid notes turn real additions into `unknown`; they can never produce a
> false 0% AI conclusion. Follow-up root-cause work fixed the capture/transport
> path itself: Codex `apply_patch` payloads now yield every concrete patch path;
> setup uses the personal plugin as the sole Codex hook owner and removes legacy
> duplicate `~/.codex/hooks.json` handlers; repeated hooks preserve the original
> session start/model; normal note export requires an exact commit blob and
> intersects only the commit's additions, so stale index ranges cannot leak into
> another commit. Missed historical capture has an explicit `notes attest-ai`
> recovery path whose lines are visibly `claimed`, and team reports break AI
> lines down by evidence strength. The local CLI was reinstalled and `tellur
> connect --no-login` installed the missing post-commit/pre-push hooks plus notes
> refspec. Added unit/integration coverage for patch-path extraction, live patch
> attribution, session preservation, exact note export, single-owner Codex setup,
> and claimed recovery. README and adapter mechanics updated. Open as PR #50.
> Workspace total: 344 Rust tests, all green. Codex review follow-up: team
> reports now exclude merge commits, preventing a base-branch merge's
> first-parent diff from being counted as new PR code; covered by a real
> base/feature/merge history integration test.
>
> **2026-07-12 — Team hub JTBD overview + responsive governance navigation.**
> On `feat/team-hub-jtbd-ux`, started the evidence-based hub UX pass for the SMB
> and corporate jobs in `TEAM_DASHBOARD_UI.md`. The old Overview led with generic
> event/session counters in five cards (overflowing the captured desktop
> viewport) and hid the entire primary nav below 768px. Rebuilt it as a calm,
> decision-first governance workspace: explicit review-gap headline and CTA,
> four restrained metrics (unreviewed AI lines, coverage, AI share, repos), a
> sorted repository priority queue that drills into evidence, evidence-volume
> context, freshness, clearer load/empty states, and restrained/reduced-motion-
> safe transitions. Mobile now uses a scrollable primary nav instead of removing
> it. Updated English/Dutch copy and the E2E contract, including a 390px mobile
> navigation test. Verification: `pnpm check`, 63 Vitest tests, production build,
> 7 Playwright E2E tests, `cargo build -p tellur-server`, and
> `cargo test -p tellur-server` green. The broader hub JTBD audit remains an
> iterative program; this change deliberately lands one coherent, reviewable
> milestone instead of mixing every screen into one unsafe mega-PR.
> Review follow-up: the decision headline now distinguishes organizations with
> zero AI-attributed lines from genuinely complete review coverage. The lockfile
> also pins `anyhow` 1.0.103, resolving RUSTSEC-2026-0190 in the supply-chain gate.
>
> **2026-07-12 — Adapter harness compatibility + supply-chain gate.** On branch
> `feat/adapter-harness-compat`. Migrated the older Codex, Claude Code, and
> Copilot import paths onto the shared tolerant JSON stream contract. All three
> now accept JSONL, top-level arrays, single objects, and common envelope exports;
> preserve source event IDs and string/numeric timestamps; inherit wrapper
> session/model metadata; and normalize nested model/command/file concepts with
> prompt hashing and secret redaction. Claude imports now retain user/assistant
> role events and recognize Anthropic `content`-block `tool_use` records,
> including `NotebookEdit`, `Grep`, and `Glob`. Copilot preserves pre-hashed
> prompts plus completion/suggestion/language correlation fields. Added four
> regression tests covering the shared metadata contract and Codex, Claude, and
> Copilot harness variants. Full workspace: 342 tests green; clippy `-D warnings`
> green. `cargo deny check` initially exposed newly published
> `RUSTSEC-2026-0190` in locked `anyhow 1.0.102`; lockfile upgraded to 1.0.103 and
> the gate is green. README and adapter docs updated; no CONTRIBUTING change was
> needed because the workflow and repository structure did not change.
>
> **2026-07-12 — Developer GUI compatibility audit.** Continued on
> `feat/adapter-harness-compat` before a PR existed. Checked the implementation
> against current official VS Code, Cursor, Windsurf, and JetBrains contracts.
> Fixed the VS Code-family extension so it requires Workspace Trust, runs as a
> workspace extension beside local/remote repos, maintains one watcher per
> multi-root folder (including runtime add/remove), and correctly auto-detects
> VS Code/Cursor/Windsurf when setup has not written an explicit source. The old
> package defaults had masked host detection, so manual Cursor/Windsurf installs
> could be mislabeled `vscode-ai`; regression tests now cover all three hosts.
> Expanded the JetBrains install range from builds 241–242 to 241–253
> (2024.1–2025.3). Verification: VS Code compile + 7 unit tests + 2 real Extension
> Host tests + VSIX package; JetBrains 3 tests + `buildPlugin`/headless IDE boot on
> JDK 17. The pinned IntelliJ Gradle Plugin 2.0.1 cannot resolve the renamed
> 2025.3 Community distribution for Plugin Verifier, so a deliberate build-tool
> upgrade + cross-version verifier matrix is recorded as follow-up. Added
> `docs/GUI_COMPATIBILITY.md`; README/adapter/plugin docs updated.
> Review follow-up: VS Code virtual workspaces are now explicitly unsupported,
> matching the filesystem-backed checkout requirement, and Codex envelope imports
> inherit wrapper `sessionId` values even when no `session_meta` event is present.
>
> **2026-06-22 — Maintainability refactor: decomposed the four monolith files.**
> On branch `refactor/decompose-monoliths`. Behavior-preserving structural split,
> no functional change. `crates/cli/src/main.rs` (4879 lines) → a 190-line
> dispatcher plus focused command modules (`cli`/`repo`/`inspect`/`capture`/
> `maintain`/`policy`/`push`/`notes`/`connect`/`setup`/`hooks`/`serve` + `util`/
> `git` helpers). `crates/server/src/api.rs` (2745) → an `api/` directory of
> per-domain handler modules (`common` auth-extractor/guards + `analytics`/`repos`/
> `sessions`/`exports`/`rbac`/`policies`/`sso`/`device`), re-exported from `api/mod.rs`
> so `crate::api::*` (router + tests) is unchanged. Both storage backends
> `storage/sqlite.rs` (3006) + `storage/postgres.rs` (2251) → `storage/{sqlite,
> postgres}/` directories: `mod.rs` keeps the connection/pool, shared query helpers
> and the schema migration and exposes a **thin delegating `impl Store`**, with the
> per-domain SQL in submodules (`orgs`/`repos`/`events`/`policy`/`audit`/
> `auth_sessions`/`jobs`/`scim`/`schema`). SQL bodies and handler logic moved
> verbatim. `storage::chain` is now `pub(crate)`; the now-dead `use storage::Store
> as _` shim in `build_state` was removed; two stray untracked macOS duplicate files
> (`github_webhook 2.rs`) that broke local clippy were deleted. Verified end-to-end
> against a live Postgres (`TELLUR_TEST_DATABASE_URL`): `cargo fmt` +
> `clippy --all-targets --all-features -D warnings` + `test` (337 workspace tests) +
> `cargo deny check` all green.

> **2026-06-16 — GitHub App repo discovery + notes harvester (proposal P3).** On
> branch `codex/github-app-notes-harvester`. Added `POST /webhook/github` for
> GitHub App webhooks. The endpoint requires `TELLUR_GITHUB_WEBHOOK_SECRET` and
> verifies `X-Hub-Signature-256` before parsing the body. Tenant binding is
> explicit: `tellur-server admin set-github-installation --org <org>
> --installation-id <id> --account <login>` writes `github_installation`, and
> unsigned/unmapped deliveries cannot create repos. `installation_repositories`
> refreshes installed repo discovery; `push` syncs installed repos and source
> templates, fetches the current `refs/notes/ai` tree through a GitHub App
> installation token, finds the note blob for each pushed commit, parses the Git
> AI authorship note, and appends a compact `github.note.harvest` event to the
> repo chain. `github_note_harvest` enforces idempotency per
> `(org, repo, commit)`, so webhook redelivery does not double-count. New schema
> v20 tables exist in SQLite + Postgres. Tests: targeted
> `cargo test -p tellur-server --test github_webhook` covers valid HMAC,
> auto-provision/source sync, note import, redelivery idempotency, and invalid
> signature rejection. Remaining: P4 PR risk-report Check Runs; live GitHub App
> E2E with real webhook delivery.
>
> **2026-06-13 — External code-review hardening (triaged).** On branch
> `fix/review-hardening`. Acted on an external full-codebase review — but most of
> its P1/P2 findings did **not** survive verification against the code, so only the
> genuinely valid items were fixed. **Fixed:** (1) `save_push_state` now writes to a
> per-process temp file (`push_state.<pid>.<nanos>.tmp`) then `rename`s (atomic; a
> crash mid-write can no longer truncate `push_state.json` and silently reset the
> high-water mark — and a per-process name keeps two concurrent pushes, e.g. the
> `connect --background` timer overlapping a pre-push, from clobbering each other's
> temp and failing `rename`; **Codex P2 on PR #45**); (2) `resolve_hub` no
> longer indexes-then-`unwrap()`s the single saved host (uses a two-`next()` match so
> a future refactor can't make it panic); (3) the OIDC discovery-cache `Mutex` locks
> are now poison-resilient (`unwrap_or_else(PoisonError::into_inner)`); (4) added the
> missing unit tests for `crates/adapters/src/sanitize.rs` (prompt-key hashing,
> secret redaction vs clean-text passthrough, recursion, `first_prompt_hash`). 333
> workspace tests, clippy `-D warnings` + cargo-deny green. **Rejected as false
> positives** (verified against the code): the "cookie prefix-matching" bug
> (`strip_prefix("name=")` already anchors on the `=`, so `tellur_session_evil=`
> does not match `tellur_session=`); the CLI setup `as_object_mut().unwrap()`
> "panics" (every call is guarded by a preceding `is_object()`/`is_array()` check
> that rewrites a non-object to empty, or runs on a freshly-built value); and the
> "missing tests" claims for `import.rs` (12 tests), `claude_code/codex/cursor.rs`,
> `remap`, and `webhook.rs` (all already have `#[cfg(test)]`). **Rejected as
> documented design tradeoffs / out-of-scope** (not bugs): Postgres `NoTls` (run
> behind a TLS proxy — threat-model documented; configurable TLS is a follow-up),
> OIDC ID-token signature not locally verified (deliberate per OIDC Core §3.1.3.7,
> documented), in-memory rate limiter + `Mutex<Connection>` SQLite (single-node
> zero-config design; multi-replica needs Redis/PG — large follow-ups), no SQLite
> migration framework, and splitting the CLI monolith. `auth_denied_total` is **not**
> dead — it's exposed at `/metrics` by design.
>
> **2026-06-13 — GitHub App installation tokens for the source proxy (proposal
> P2).** On branch `feat/github-app-tokens`. The private-repo blob proxy can now
> authenticate to **GitHub** with short-lived **App installation tokens** instead
> of a stored PAT. New `crates/server/src/github_app.rs`: `GithubAppConfig::from_env`
> (`TELLUR_GITHUB_APP_ID` + `TELLUR_GITHUB_APP_PRIVATE_KEY` or `…_PRIVATE_KEY_FILE`,
> optional `TELLUR_GITHUB_API_BASE` for GHES) enables the App only when fully set;
> `build_app_jwt` signs the App JWT (RS256, `iat`-60s/`exp`+9min) via the new
> `jsonwebtoken` dep; `GithubAppApi` trait (mockable; `HttpGithubAppApi` over
> ureq/rustls) resolves the installation (`GET /repos/{o}/{r}/installation`) and
> mints a per-repo `Contents:read` token (`POST /app/installations/{id}/access_tokens`);
> `GithubAppRuntime` caches tokens per `(owner,repo)` until ~5 min before expiry.
> `api::resolve_source_token` prefers the App token for GitHub repos (parsed from
> the raw/contents template by `github_owner_repo`) and falls back to the stored
> PAT for GitLab/Bitbucket/self-managed, or if minting fails. `AppState.github_app`
> wired in `build_state`. The SSRF allowlist, https-only, 2 MB cap, and
> token-never-returned contract are unchanged (only the token *source* changes).
> Tests: 5 `github_app` unit tests (JWT build/sign + bad-key, owner/repo parse for
> raw + contents hosts, non-GitHub/templated skip, token cache) + 3 `source`
> integration tests (App token replaces the PAT for GitHub; non-GitHub keeps the
> PAT; no-App keeps the PAT) via a mock `GithubAppApi`; all 13 server test
> AppState constructors updated. Workspace 329 tests, clippy `-D warnings` +
> cargo-deny (jsonwebtoken/simple_asn1 licenses) green. **Not E2E-tested against a
> real GitHub App** (no App credentials in CI) — verify with a live App. Docs:
> README (source connection + hub env vars), AGENTS map, THREAT_MODEL (new outbound
> App-authed Hub→GitHub boundary + App-key secret), the operator walkthrough
> [`docs/GITHUB_APP_SETUP.md`](docs/GITHUB_APP_SETUP.md) (register → install →
> configure → verify), this file. **Codex review (PR #44):** (1) token minting
> (JWT sign + 2 sync GitHub calls on the uncached path) now runs inside the
> `run_blocking` closure alongside the fetch, so a slow GitHub response can't block
> a Tokio worker; (2) **GitHub Enterprise** is now actually supported — the host of
> `TELLUR_GITHUB_API_BASE` is threaded through `github_owner_repo`, the source SSRF
> allowlist, and the auth-header classifier, so a GHES **Contents API** template is
> recognised + fetched (a `raw.<host>` subdomain is out of scope). Superseded:
> P3 repo discovery + notes-harvester webhook is now shipped; remaining P4
> PR-check runs.
>
> **2026-06-13 — Zero-touch `tellur connect` (GitHub App proposal P1, complete).**
> On branch `feat/connect-zero-touch`. New top-level CLI command that bundles the
> one-time setup so a developer never runs a `tellur` command again. `tellur
> connect [--hub <url>]` (1) runs `tellur login` (best-effort — an unreachable hub
> downgrades to a warning, never aborts), (2) runs `tellur setup agents`, and (3)
> installs two **chained, non-clobbering git hooks**: `post-commit` → `tellur notes
> export` (refresh `refs/notes/ai` from the local index) and `pre-push` → `tellur
> push` (events → hub) + `tellur notes push "${1:-origin}"` (notes ride the push),
> with a `TELLUR_CONNECT_PREPUSH` recursion guard (the nested notes-push `git push`
> re-enters pre-push) and `|| true` everywhere so the hub never blocks a commit or
> push. Also reuses `notes install-config` for the fetch refspec. Hooks are spliced
> into a marker-delimited block (`# >>> tellur connect (managed) >>>`) so a
> pre-existing user hook is preserved; re-install is idempotent. (4) **`--background`
> installs an always-on per-user push service** (new `crates/cli/src/service.rs`):
> launchd `~/Library/LaunchAgents/dev.tellur.push.<id>.plist` (macOS) or systemd
> `--user` `tellur-push-<id>.{service,timer}` (Linux) that runs `tellur push` every
> `--push-interval` secs (default 900) so idle machines sync between git pushes;
> opt-in (a login-start service is intrusive); unsupported platforms error clearly.
> Activation (`launchctl`/`systemctl`) is best-effort and skipped under
> `TELLUR_CONNECT_NO_ACTIVATE` (test-only). `--status` shows hooks + notes config +
> service; `--remove` excises our hook block (keeping the user's hook), deletes our
> own hooks, unsets the notes fetch config, and tears down the service.
> **Deliberate deviation from the proposal:** no `remote.<remote>.push` refspec — it
> would override git's default of pushing the current branch and push *only* notes;
> the pre-push hook covers notes-push instead. Tests: 4 new CLI integration tests
> (hook+notes-config install incl. exec bit; chaining + idempotent re-install +
> `--remove` restores the user's hook; `--background` writes+removes the service
> file under a sandboxed `HOME`; no-remote run does not create a phantom remote);
> full CLI suite 33 green; workspace clippy `-D warnings` + cargo-deny green.
> **Codex review (PR #43):** the notes fetch refspec is now written only when the
> target remote already exists — writing `remote.<remote>.fetch` in a freshly
> `git init`'d repo materialised a phantom `origin` that broke a later `git remote
> add origin`. Docs: README (*Zero-touch setup* section + CLI reference), AGENTS
> architecture map, this file. Superseded: P2 installation tokens and P3 repo
> discovery + notes-harvester webhook are now shipped; remaining P4 PR-check runs.
>
> **2026-06-12 — Prompt excerpts (opt-in) + dynamic session timeline.** On branch
> `feat/timeline-prompts`. Made the dead `redaction.store_prompt_excerpt` flag
> real: when a repo opts in, the `UserPromptSubmit` hook stores a **secret-redacted,
> length-bounded (`PROMPT_EXCERPT_MAX=600`) `prompt_excerpt`** in the `user.prompt`
> event payload (alongside the existing hash); it rides through `tellur push` →
> hub (re-redacted on ingest) → the session timeline. **Default stays `false`**
> (privacy); excerpts only apply to activity captured after opting in. Redesigned
> the hub **session timeline** (`SessionDetail.svelte`) from a flat list into a
> dynamic vertical timeline: sticky summary (events / duration / files / prompts +
> actor filter chips), category filter chips + text search, per-event nodes
> color-coded by category (prompt/file/command/tool/test/git/session/policy) with
> a prompt bubble, command exit badges, and an expandable raw-payload `<details>`.
> Pure helpers in `lib/timeline.ts` (eventCategory / eventDetail / sessionStats /
> formatDuration), unit-tested. Tests: CLI prompt-excerpt unit + 8 timeline vitest;
> full suite 148 server + 11 CLI + 63 vitest + 5 E2E green. Review-coverage left as
> a no-op metric by design (review marking does not belong in the hub).

> **2026-06-12 — Proposal: zero-touch provenance + GitHub App.** On branch
> `docs/github-app-proposal`. Design doc `docs/proposals/GITHUB_APP.md` (no code).
> Product principle "after one-time install the developer never touches the
> terminal": a `tellur connect` umbrella installing git hooks (`post-commit`
> notes, `pre-push` flush) + a background pusher + auto `refs/notes/ai` push (Part
> A, provider-agnostic), and an optional GitHub App (Part B: installation-token
> source access replacing the stored PAT, repo discovery, a `push`-webhook
> **notes harvester** syncing commit-level attribution GitHub→hub, PR-check runs).
> Includes the data-location ground truth (traces/index are gitignored → only
> commit-level notes reach git, so the App harvests that slice + serves source,
> never the rich stream) and threat-model deltas (inbound webhook HMAC, App
> key/webhook secret, notes-push privacy). Order: P1 zero-touch client first.

> **2026-06-12 — A12 source connection: dashboard UX + private-repo blob proxy.**
> On branch `feat/source-connection`. Two parts. **(1) Settings UX:** an admin
> **Source connection** card on the repo screen (`SourceConnection.svelte`) —
> pick GitHub/GitLab/Bitbucket + `owner/repo` + branch and it generates the
> link/raw templates (pure `buildTemplates`, unit-tested), with a private toggle,
> token field, and Advanced template editor; backed by `GET .../repos/{repo}/source`
> (admin; returns `token_configured`, never the token) + the extended `PUT`
> (token preserve/replace/clear semantics, audited without the value). Also a
> `tellur-server admin set-repo-source` CLI. **(2) Private-repo blob proxy:**
> `GET .../repos/{repo}/blob?path=` (viewer+) fetches raw bytes server-side via a
> new `source` module — SSRF host allowlist (raw.githubusercontent.com,
> api.github.com, gitlab.com, bitbucket.org, api.bitbucket.org; userinfo/ports
> rejected), `https`-only, 2 MB cap, per-provider auth header (Bearer /
> PRIVATE-TOKEN / GitHub raw Accept); token stored (`repo_source.source_token`,
> schema v19, both backends) and never returned. The FileView gutter routes
> through the proxy when `source_proxy` is set (private), else fetches the
> provider directly (public). Token is `#[serde(skip)]` on `RepoSource` so it
> can't leak. Tests: 5 source integration specs + 4 `source`-module unit tests +
> Postgres surface (token round-trip) + 4 `buildTemplates` vitest; full suite 185
> Rust + 55 vitest green; removed two stray ` 2.ts` duplicate files. Docs:
> README, THREAT_MODEL (proxy = new SSRF-guarded outbound Hub→provider boundary +
> stored-token posture; roadmap item closed).

> **2026-06-11 — Seamless hub coupling: `tellur login` + `tellur push`.** On
> branch `feat/cli-login-push`. Closes the gap between local capture and the hub
> dashboard. **`tellur login`** runs an RFC 8628 device-authorization grant
> against the hub (`POST /v1/device/authorize` + `/v1/device/token`, schema v18
> `device_auth` table on both SQLite + Postgres): the CLI prints a short code,
> opens the hub's server-rendered approval page (`GET /auth/device`, session-
> gated, bounces through SSO via a validated return cookie; `POST
> /auth/device/decision`), and stores the minted member token at
> `~/.config/tellur/hosts.json` (`0600`). The token is minted at poll time from
> the member's *current* state and delivered at most once (row consumed under an
> advisory-locked tx). **`tellur push`** forwards locally-captured events to the
> ingest API with a per-`(hub, org, repo)` high-water mark in
> `.tellur/push_state.json` — incremental + idempotent, `--dry-run`/`--reset`,
> hub/org/token defaulting to the stored login. CLI HTTPS via ureq's `tls`
> feature (reuses the rustls/webpki crates the server already pulls in). Tests:
> 6 device integration specs + the Postgres surface test + 8 CLI unit tests
> (high-water-mark slice logic, actor/host mapping, path-segment encoding, and
> the device-token `access_token`→`token` round-trip); full e2e push smoke
> verified against a live hub. Docs: README (new *Connect a developer* section),
> THREAT_MODEL (device-auth boundary + CLI credential storage). `tellur logout`
> added. Requires SSO enabled (device login has no identity otherwise).
> Post-merge fixes (own PRs off main): percent-encode org/repo push path
> segments (#33, Codex P2); accept RFC 8628 `access_token` in the login response
> so `tellur login` completes (#34).

> **2026-06-10 — Dashboard E2E (Playwright).** On branch `feat/e2e`. Browser
> end-to-end tests drive the **real production bundle** in Chromium with the
> `/v1` API mocked per-test via route interception — the SPA can't complete the
> cookie/SSO login flow headlessly, so this is honest *frontend* E2E (routing,
> rendering from fixtures, admin role-gating, live EN/NL switch, theme toggle,
> command-palette navigation), not full-stack. `crates/server/ui/e2e/` +
> `playwright.config.ts` (webServer builds + `vite preview`s under `/app/`); 5
> specs, all green locally. New CI job **dashboard E2E (Playwright)** installs
> chromium and runs them. The full-stack/hub-proxy paths stay out (would need a
> test-only auth bypass — deliberately not added). Remaining follow-up: A12
> hub-proxy `/blob` variant (private repos without provider CORS).

> **2026-06-10 — Full dashboard i18n (English + Dutch).** On branch `feat/i18n`.
> Every user-facing string in the SPA resolves through a message catalog: pure
> `lib/i18n.ts` (`translate(locale, key, vars)` with `{var}` interpolation +
> en-fallback) + reactive `lib/i18n.svelte.ts` (`t()` reads a `$state` locale
> rune, so switching language re-renders the whole UI live). Wired through every
> screen + component (AppShell nav/topbar, CommandPalette, App, Trend, Overview,
> Repos, RepoDetail, FileView, Sessions, SessionDetail, Audit, Exports, Policies,
> People); palette labels became i18n keys (`labelKey`/`hintKey`, resolved before
> fuzzy-filtering). A topbar **EN/NL** switcher persists to localStorage and sets
> `<html lang>`. Two complete hand-authored locales (no machine-fill); a vitest
> **catalog-parity** test fails if the en/nl key sets diverge. Verified: SPA 51
> vitest (i18n + commands incl. parity) + svelte-check (0/0) + build; `cargo build
> --features dashboard` embeds. Remaining follow-ups: Playwright E2E; A12
> hub-proxy `/blob` variant.

> **2026-06-10 — A12 complete: inline source gutter (client-fetch variant).** On
> branch `feat/source-gutter`. Completes A12 while keeping the hub metadata-only:
> the **browser** fetches raw bytes straight from the provider (never via the
> hub), so there's no hub egress/SSRF/secret-handling. `repo_source` gains a
> `raw_template` column (schema v17, SQLite + PG); `RepoSource { link, raw }`,
> `PUT .../repos/{repo}/source` takes both (each https-validated). The File view
> gains an opt-in **Show source** toggle that fetches the raw file once
> (`credentials: omit`, 2 MB cap, graceful CORS/auth fallback to the links) and
> renders each attributed range's actual lines in an origin-coloured gutter. The
> `/app` CSP `connect-src` is widened to a small all-list
> (raw.githubusercontent.com, gitlab.com, bitbucket.org) so the client fetch
> works for common providers; others fall back to links. Pure `rawUrl`/`sliceLines`
> unit-tested. Verified: server tests (link+raw set/clear/surface, https+admin
> validation; PG parity) + SPA 44 vitest + svelte-check + build; clippy -D
> warnings + cargo-deny green. The hub-proxy `/blob` variant stays intentionally
> unbuilt (only needed for private repos without provider CORS). Remaining: full
> i18n, Playwright E2E.

> **2026-06-10 — A12 opt-in source links (provider deep-link variant).** On
> branch `feat/source-links`. Implements the safe half of A12 from the dashboard
> plan — "Git-provider link/fetch" — **without** the hub ever storing or proxying
> source. A per-repo `repo_source` template (schema v16, SQLite + PG) holds only
> a URL template; `set_repo_source`/`get_repo_source` + `PUT
> /v1/orgs/{org}/repos/{repo}/source` (admin, validates `https://`, ≤2048 chars,
> audited). The attributions read now returns `source_template`, and the File
> view renders a per-range **View ↗** deep link (`{path}` (URL-encoded per
> segment) /`{start}`/`{end}` substituted, https-guarded client-side too).
> Verified: server tests
> (`dashboard_api` set/clear/surface + https+admin validation; PG parity for
> get/set), SPA 40 vitest (new `source` helper tests) + svelte-check + build;
> clippy -D warnings + cargo-deny green. The **inline full-source gutter** (hub
> serving bytes; option 2 of A12) stays deferred — it needs a provider proxy with
> SSRF allow-listing + secret redaction, its own careful design. Remaining
> follow-ups: full i18n, Playwright E2E, A12 inline-source proxy.

> **2026-06-10 — Sealed-checkpoint audit retention.** On branch
> `feat/audit-checkpoint`. The audit hash chain can now be minimised without
> losing tamper-evidence. `audit_head` gains `sealed_hash`/`sealed_count`
> (schema v15, SQLite + PG): `seal_audit_before(cutoff)` deletes entries with
> `ts < cutoff` and records the pruned prefix's tip hash + length as a
> checkpoint. `verify_audit_chain` now **seeds** the walk from that checkpoint
> (via a new `base` arg on the shared `chain::verify`), so the retained suffix —
> or an empty chain — still verifies against the head and truncation stays
> detectable. The retention loop gains a second window: `RetentionPolicy { jobs_days,
> audit_days }`, driven by `TELLUR_AUDIT_RETENTION_DAYS` (default 0 = keep). The
> event chain is deliberately untouched (events are the provenance data). Verified:
> `tests/jobs.rs` seal test (partial seal → remainder verifies; append-after-seal;
> full seal → empty chain verifies) + PG parity; full workspace tests, clippy -D
> warnings + cargo-deny green. Remaining follow-ups: full i18n, Playwright E2E,
> A12 (opt-in full-source gutter).

> **2026-06-10 — Data retention / lifecycle hygiene.** On branch
> `feat/retention`. A background maintenance loop (`jobs::spawn_maintenance`,
> hourly) prunes **transient** data only: expired browser sessions and stale
> OIDC login transactions are always cleaned up; finished (`completed`/`failed`)
> jobs are pruned when `TELLUR_RETENTION_DAYS > 0` (default 0 = keep). New Store
> methods `prune_expired_sessions` + `prune_finished_jobs(cutoff)` (SQLite + PG);
> `run_maintenance_once` is pure-ish and unit-tested. The tamper-evident **event
> and audit chains are never pruned** (that would break verification; a future
> "sealed checkpoint" design is the path for compliance-driven minimisation).
> Retention is read from env in `run()` (not `Config`), so no test-literal churn.
> Verified: full workspace tests (new retention test in `tests/jobs.rs` + PG
> parity for both prune methods), clippy -D warnings + cargo-deny green; PG tests
> pass against a local Postgres. Remaining: full i18n, Playwright E2E, A12
> (opt-in full-source gutter), sealed-checkpoint audit retention.

> **2026-06-10 — Evidence exports (A13 + org evidence pack).** On branch
> `feat/evidence-exports`. The durable-job queue gains a `params` column (schema
> v14, SQLite + PG) so jobs can carry arguments. **A13**: per-repo SLSA/SPDX now
> run as durable jobs — `POST /v1/orgs/{org}/repos/{repo}/export/slsa|spdx`
> (admin / per-repo-admin) enqueue `KIND_EXPORT_SLSA|SPDX` with `{repo_id,
> repo_url?, commit?}` params; the synchronous `GET` forms stay. **Evidence
> pack**: `POST /v1/orgs/{org}/export/evidence` (admin) enqueues
> `KIND_EXPORT_EVIDENCE`, a worker job that bundles every repo's SLSA provenance
> + the latest compliance snapshots + the audit chain's verification state into
> one downloadable result. The Exports console gets a primary "Evidence pack"
> action (reuses the existing job-poll + download). Verified: full workspace
> tests (new `evidence_pack_*`, `per_repo_slsa_export_*` incl. admin-only; jobs
> param round-trip; PG parity), SPA 36 vitest + svelte-check (0/0) + build;
> clippy -D warnings + cargo-deny green; `cargo build --features dashboard`
> embeds. Remaining: full i18n, Playwright E2E, A12 (opt-in full-source gutter),
> backup/retention.

> **2026-06-10 — Dashboard: composed `/overview` (A9) + density toggle.** On
> branch `feat/dashboard-overview-density`. Two bundled items. **A9**: new
> `GET /v1/orgs/{org}/overview` (viewer+, rate-limited, off-runtime) returns the
> landing screen in one round-trip — org totals, org-wide AI share + review
> coverage (folds attribution across repos), a 30-day activity series, repos
> **ranked by review gap** (most unreviewed AI lines first), and a recent-activity
> feed; the Overview screen now makes a single fetch and surfaces the review-gap
> ranking + an AI-reviewed KPI (warns under 50%). **Density toggle**: a
> Cozy/Compact control in the topbar drives `--row-pad-*` / `--card-pad` spacing
> tokens via `<html data-density>`, persisted to localStorage and applied before
> first paint; threaded through every table/card. Verified: full workspace tests
> (new `overview_*` coverage incl. tenant scoping + review-gap ranking; PG parity
> unaffected), SPA 36 vitest + svelte-check (0/0) + build; clippy -D warnings +
> cargo-deny green. Remaining follow-ups: full i18n, Playwright E2E, A12 (opt-in
> full-source gutter), A13 (job-backed SLSA/SPDX), backup/retention.
>
> **2026-06-10 — Team dashboard D5 (polish: command palette + theme + a11y).**
> On branch `feat/dashboard-d5`. Cross-cutting polish on the SPA. **Command
> palette** (⌘K / Ctrl-K): role-aware, org-scoped quick nav with subsequence
> fuzzy match, full keyboard operation (arrows/enter/esc) and an accessible
> `role="dialog"` listbox; pure `buildCommands`/`filterCommands` unit-tested.
> **Theme**: system/light/dark preference (cycled from the topbar), persisted to
> localStorage and applied to `<html data-theme>` before content paints; pure
> `resolveTheme`/`nextPref`/`normalizePref` unit-tested. **A11y**: skip-to-content
> link, `aria-current="page"` on the active rail item, and a global
> `:focus-visible` ring. Verified: SPA 34 vitest + svelte-check (0 errors, 0
> warnings) + build (~31KB gzip); `cargo build --features dashboard` embeds the
> bundle. **Deferred from the D5 plan (honest scope):** density toggle, full i18n
> (needs a string-catalog framework), Playwright E2E (needs a server+SSO harness
> in CI), and a dedicated performance pass — tracked as follow-ups, not shipped
> here. With this, the team-dashboard plan (D0–D5 core) is delivered.
>
> **2026-06-10 — Team dashboard D4 UI (Policies + People & Access).** On branch
> `feat/dashboard-d4-ui`. Two admin-only SPA screens on the D4 API. **Policies**:
> per-repo compliance from `GET .../policies/compliance` — KPI band (repos
> evaluated, open violations, severity split), a most-at-risk-first table
> (AI ranges, violations, severity chips high=risk/medium=warn/low=muted, policy
> version, last run), a "Re-evaluate" action that enqueues the durable job and
> polls it to completion, and a guided empty state when no `default` policy has
> run. **People & Access**: an SSO/SCIM status band (OIDC issuer, SCIM token age,
> active/total members, SSO-bound count, group count — health only), a members
> table (role chip, email, SSO-bound, active/deactivated; inactive sorted last),
> and a groups table (displayName → derived role). Admin-only rail items
> (Policies, People & Access) now active; routes `/app/orgs/:org/policies` and
> `.../people`. Verified: SPA 22 vitest + svelte-check (0 errors) + build
> (~29KB gzip); `cargo build --features dashboard` embeds the new bundle. Next:
> D5 polish (command palette, density/theme/i18n, a11y audit, E2E).
>
> **2026-06-10 — Team dashboard D4 API (policy compliance + People & Access).**
> On branch `feat/dashboard-d4`. API-first; the Policies + People & Access UI
> follows in a separate D4-UI PR. New admin-only, tenant-scoped endpoints:
> **A8 policy compliance** — `POST /v1/orgs/{org}/policies/compliance` enqueues a
> durable job that evaluates the org's `default` policy (via the core
> `PolicyEngine`) over every repo's attribution and persists timestamped
> snapshots per `(org, repo, policy version)`; `GET .../policies/compliance`
> reads the latest snapshot per repo (`evaluated` flag + per-repo violation
> counts by severity, Critical folded into High). **A2** `GET .../members`
> (role, email, sso_bound, active). **A11** `GET .../groups` — session-auth
> mirror of `/scim/v2/Groups` (members + derived `maps_to_role`) so the SPA
> never holds a SCIM token. **A10** `GET .../sso-status` — OIDC/SCIM health
> (issuer, scim-token age, member/sso/group counts; no secrets). Schema → v13
> (`compliance_snapshot` table); Store gains `list_members`,
> `scim_token_created_at`, `put_compliance_snapshot`, `latest_compliance`;
> worker gains `KIND_COMPLIANCE`. Verified: full workspace tests (new
> `dashboard_api` compliance + people coverage incl. admin-only/tenant + a
> policy-eval round-trip via `process_one`; PG parity), clippy -D warnings +
> cargo-deny green; PG tests pass against a local Postgres. Next: D4 UI
> (Policies + People & Access screens), then D5 polish.
>
> **2026-06-10 — Team dashboard D3 (audit read + exports).** On branch
> `feat/dashboard-d3`. API-first then UI. New admin-only, tenant-scoped read
> endpoints: `GET /v1/orgs/{org}/audit[?actor=&action=&range=&before=&limit=]`
> (A7 — paginated, newest-first read of the tamper-evident audit log; keyset
> cursor via `before=<seq>`; on the first page only, `chain_intact` reports
> whether the global hash chain still verifies — an O(n) check skipped on later
> pages) and `GET /v1/orgs/{org}/jobs` (durable-job history for the Exports
> table; results not inlined — poll `.../jobs/{id}`). Store gains `list_audit`
> (dynamic but fully parameterised filter) + `list_jobs` (SQLite + Postgres
> parity). SPA adds an **Audit log** screen (filterable table, chain-verified
> badge, load-more pagination) and an **Exports** screen (start events/audit
> exports, live job-status polling, download a completed job's JSON result);
> both are admin-only and hidden from the rail for non-admins (the API enforces
> the role too). Routes `/app/orgs/:org/audit` and `.../exports`. Verified: all
> server tests pass (incl. `dashboard_api` A7/jobs coverage + PG parity), SPA 20
> vitest + check + build (~26KB gzip); clippy -D warnings + cargo-deny green; PG
> tests pass against a local Postgres. Next: D4 (policy compliance snapshots A8 +
> People & Access A2/A10/A11).
>
> **2026-06-08 — Team dashboard D2 (evidence: attribution + sessions).** On
> branch `feat/dashboard-d2`. API-first then UI. New read endpoints (viewer+,
> tenant-scoped): `GET /v1/orgs/{org}/repos/{repo}/attributions[?path=]` (A4 —
> read stored line-level attribution; metadata only, no source text), and
> sessions (A6) — `GET /v1/orgs/{org}/sessions[?repo=&actor=&range=&limit=]`
> (events grouped by `session_id`: count, first/last ts, distinct actors/repos)
> + `GET /v1/orgs/{org}/sessions/{id}` (events oldest-first for replay). Store
> gains `list_sessions` + `session_events` (group_concat / string_agg for the
> per-session distinct facets; SQLite + Postgres parity). SPA adds a **File
> provenance view** (metadata-first gutter: per-range origin colour, agent/model,
> confidence, reviewed-by — explicitly no source text), an attributed-files list
> on Repo detail, a **Sessions** list, and a **Session replay** timeline; routes
> `/app/orgs/:org/repos/:repo/files/:path*`, `.../sessions[/:id]`; Sessions nav
> active. Verified: 269 workspace tests (incl. `dashboard_api` A4/A6 coverage +
> PG parity), SPA 18 vitest + check + build (~22KB gzip); clippy -D warnings +
> cargo-deny green; PG tests pass against a local Postgres. Next: D3 (audit read
> A7 + Exports screen).
>
> **2026-06-08 — Team dashboard D1 (activity + repositories).** On branch
> `feat/dashboard-d1-api`. API-first then UI, one PR. New read endpoints
> (viewer+, tenant-scoped): `GET /v1/orgs/{org}/activity?range=&group_by=type|actor`
> (A1 — daily event time-series; `since` filter + day bucket, SQLite `substr` /
> Postgres `left`), and `GET /v1/orgs/{org}/repos/{repo}` (A3 — single-repo
> summary: event count, contributors, last activity, and **line-level AI share +
> review coverage**). The review-gap math is a pure, unit-tested `review` module
> implementing decision §12.1 (AI range reviewed iff explicit human reviewer ≠
> producing agent, a `reviewed_at`, and passing tests where tests were run;
> documented approximation: no per-range modification ts). Store gains
> `activity_by_day` + `repo_facts` (SQLite + Postgres). SPA (Svelte) adds an
> Overview activity **trend** (bespoke SVG bars), a **Repositories** list, and a
> **Repo detail** screen (AI-share / review-coverage KPIs, contributors), with
> org-scoped routes `/app/orgs/:org/repos[/:repo]`. Verified: 266 workspace
> tests (incl. `review` unit tests + `dashboard_api` integration + PG parity),
> SPA 16 vitest + check + build (~20KB gzip); clippy -D warnings + cargo-deny
> green; PG tests pass against a local Postgres. Next: D2 (attribution read A4 +
> sessions A6 → file provenance gutter + session replay).
>
> **2026-06-08 — Team dashboard D0 (foundation).** On branch
> `feat/dashboard-d0-foundation`, executing `docs/proposals/TEAM_DASHBOARD_UI.md`
> phase D0. The hub now serves a real **team dashboard SPA at `/app`**: Svelte 5
> + TypeScript source in `crates/server/ui/`, built with Vite, embedded into the
> binary via `rust-embed` behind the default-on `dashboard` Cargo feature, served
> same-origin with SPA client-routing fallback (`/app/*` → `index.html`; unknown
> `/v1` still 404s; hashed assets cached immutably, HTML `no-cache`). `build.rs`
> creates an empty `ui/dist` so a plain `cargo build` still compiles (serving a
> placeholder); the new `dashboard` CI job and the Docker build compile the real
> SPA and embed it. The SPA ships AppShell (rail + topbar), org-scoped routing
> (`/app/orgs/:org/...`), `/v1/me` bootstrap + 401→`/auth/login?return=` redirect,
> the design tokens from the plan (Tellur-green accent, dark default + light), and
> an **Overview** screen on the existing `GET /v1/orgs/{org}/dashboard` payload
> (KPIs, repos, recent activity, event types). FSL like the rest of the server.
> Verified: Rust `tests/dashboard_routes.rs` (4) + SPA vitest (12, router/format)
> + `pnpm check`/`build` clean; workspace clippy + cargo-deny green; SPA bundle
> ~18KB gzip. Next: D1 (activity time-series A1 + repo summary A3 + repos
> screens). Decisions §12 of the plan are resolved.
>
> **2026-06-06 — Hub: durable jobs + SCIM groups + dashboard coupling.** On
> branch `feat/server-jobs-scim-groups-dashboard` (one PR, three features).
> **Durable job queue** (`jobs` module + `job` table, schema v12): org exports
> are now enqueued (`POST .../export/events|audit` → 202 + `job_id`) and run by a
> background worker (`spawn_worker`; `process_one` is deterministic for tests),
> polled at `GET .../jobs/{id}` (admin, tenant-scoped). Atomic claim via SQLite
> IMMEDIATE / Postgres `FOR UPDATE SKIP LOCKED`. **SCIM Groups** (`/scim/v2/Groups`
> CRUD + `scim_group`/`scim_group_member`): a group `displayName` of
> `tellur-admin|contributor|viewer` drives members' org role, recomputed on
> membership/rename/delete; mutations audited. **Dashboard coupling**:
> `GET /v1/orgs/{org}/dashboard` (viewer; session-cookie or token) returns the
> org rollup + a recent-activity feed (`recent_org_events`); `web/index.html`
> gains a hub mode (`?hub=&org=`, `credentials:include`, served same-origin to
> avoid CORS). SQLite + Postgres parity throughout. Verified: new
> `tests/jobs.rs`, SCIM group + dashboard tests, updated export tests (202+poll);
> 252 workspace tests; clippy -D warnings + cargo-deny green; PG tests pass
> against a local Postgres. Remaining hub work: backup/retention; a fuller
> dashboard UI.
>
> **2026-06-06 — Tier 1 B6 (enterprise) — SCIM 2.0 provisioning (B6 complete).**
> On branch `feat/server-b6-scim`. New `scim` module exposes `/scim/v2/Users`
> (list/create/get/PUT/PATCH/DELETE) so an IdP can auto-provision and, crucially,
> **deprovision** hub members. Auth is a dedicated, org-scoped SCIM bearer token
> (`scim_token` table, Argon2id-hashed, minted via `tellur-server admin
> create-scim-token`); the org comes from the token, never the URL. A SCIM user
> maps to a member + SSO identity: `userName`→email, `displayName`/`name`→display
> name, optional `roles`→org role (default `viewer`), `externalId` round-tripped.
> Schema v11 adds `member.active` + `member_identity.external_id`; **all three
> auth paths (API token, session, SSO email lookup) now filter `active`**, so
> `DELETE`/`PATCH active=false` revokes a member across every credential type
> immediately. SQLite + Postgres parity. Scope: Users only — Group-based role
> sync is deferred (documented). Verified: `tests/scim.rs` (provision → list/get
> /filter, duplicate→409, bad/no token→401, deactivate→auth fails but still
> listed, PATCH reactivate+role change, tenant isolation) + `scim` unit tests +
> Postgres parity; 243 workspace tests; clippy -D warnings + cargo-deny green; PG
> tests pass against a local Postgres. **B6 (enterprise) is now complete**
> (per-repo RBAC + OIDC SSO + SCIM). Remaining hub work: queued/durable jobs.
>
> **2026-06-06 — Tier 1 B6 (enterprise) — OIDC SSO.** On branch
> `feat/server-b6-oidc-sso`. Browser single sign-on via OIDC Authorization Code
> + PKCE. New `oidc` module: PKCE (S256), authorize-URL builder, ID-token claim
> validation (`iss`/`aud`/`exp`/`nonce`) — signature integrity rests on the
> TLS-secured direct token-endpoint channel (OIDC Core §3.1.3.7), documented in
> the threat model. The IdP boundary is behind an `OidcClient` trait
> (`HttpOidcClient` over ureq/rustls; a mock drives the tests with no network).
> New routes `/auth/login`, `/auth/callback`, `/auth/logout`; the `Principal`
> extractor now accepts an API bearer token **or** an opaque, DB-backed session
> cookie (`HttpOnly`/`Secure`/`SameSite=Lax`, 8h). Schema v9 adds
> `member_identity` (email + bound OIDC subject), `oidc_login` (CSRF state →
> PKCE/nonce), and `session`; full SQLite + Postgres parity. No open
> self-registration: sign-in is limited to members provisioned by verified email
> (`tellur-server admin add-member`), with the OIDC subject bound on first login.
> Deps: `ureq`(+rustls TLS), `base64`, `sha2`; `deny.toml` now allows
> `CDLA-Permissive-2.0` (webpki-roots). Verified: `oidc` unit tests (PKCE,
> authorize URL, claim validation good/bad), `tests/oidc.rs` end-to-end via mock
> IdP (login→callback→session→/v1/me→logout, unprovisioned/unverified→403, CSRF
> state→400, SSO-disabled→404, bearer still works), Postgres parity in
> `tests/postgres.rs`; 235 workspace tests; clippy -D warnings + cargo-deny
> green; PG tests pass against a local Postgres. Next B6 slice: SCIM.
>
> **2026-06-05 — Tier 1 B6 (enterprise) — fine-grained per-repo RBAC.** On
> branch `feat/server-b6-repo-rbac`. First slice of B6: per-repo role grants on
> top of the org-level RBAC. Grants are **additive** — a member's effective role
> on a repo is `max(org_role, repo_grant)`, so a grant can elevate (e.g. an org
> viewer becomes a contributor or admin on one repo) but never reduces a member
> below their org baseline. New `repo_role` table (schema v8, both SQLite and
> Postgres backends), `Store` methods (`set/get/remove/list_repo_role(s)`,
> tenant-scoped: the repo *and* the member must belong to the org → no
> cross-tenant grants), `Role::max`, and an `effective_role` helper used by the
> write/export handlers (event + attribution ingest honour per-repo contributor
> grants; per-repo SLSA/SPDX export honours per-repo admin grants). Management
> endpoints (org-admin only, audited): `PUT/DELETE /v1/orgs/{org}/repos/{repo}/
> roles/{member}` + `GET .../roles`, plus `tellur-server admin
> grant-repo-role|revoke-repo-role|list-repo-roles`. Verified: new
> `tests/repo_rbac.rs` (grant elevates a viewer, scope is per-repo, revoke
> restores baseline, per-repo admin export, admin-only + tenant-scoped
> management) + Postgres parity in `tests/postgres.rs`; 223 workspace tests;
> clippy -D warnings + cargo-deny green; PG tests pass against a local Postgres.
> Next B6 slices: OIDC SSO, then SCIM.
>
> **2026-06-05 — Tier 1 B5 Postgres backend.** On branch `feat/server-postgres`.
> The hub now has a second storage backend: `PostgresStore`
> (`crates/server/src/storage/postgres.rs`) implements the full `Store` trait
> over an `r2d2` connection pool (sync `postgres` client, `NoTls`). Backend
> selection is by config — set `TELLUR_DATABASE_URL` and `build_state` uses
> Postgres, otherwise the embedded SQLite store (zero-config) is used. Semantics
> are mirrored exactly: same tenant scoping, the same server-side hash-chain
> recomputation, and the same head-hash + length checkpoints for truncation
> detection; the per-repo event chain and audit chain appends take a
> `pg_advisory_xact_lock` so the read-head + insert + head-update stay atomic
> across pooled connections (the Postgres equivalent of SQLite's
> `BEGIN IMMEDIATE`). New integration tests (`crates/server/tests/postgres.rs`)
> cover the whole trait surface + a tamper-detection case; they run only when
> `TELLUR_TEST_DATABASE_URL` points at a disposable DB (the test resets the
> `public` schema). CI gains a `postgres:16` service so those tests run on every
> PR. Docs (README/AGENTS/THREAT_MODEL/compose) updated; the threat model now
> records the hub↔Postgres link as a NoTls trust boundary (private network / TLS
> proxy; connection string is a secret). Verified locally against a real
> Postgres (`postgres://postgres@127.0.0.1:5433/tellur_test`): both PG tests
> pass; 218 workspace tests; clippy `-D warnings` + cargo-deny green. (Docker
> image build is verified in CI.) Next: B6 (enterprise SSO/OIDC + RBAC/SCIM).
>
> **2026-06-04 — B4 review fixes (Codex).** Addressed 3 P2 findings on PR #6:
> org event exports now carry `repo_id` per event (multi-repo context);
> `docs/THREAT_MODEL.md` updated for the policy-write + export endpoints
> (disclosure/DoS, policy bodies validated/declarative); and `README.md` now
> documents the self-hosted hub (preview) instead of saying it's unimplemented.
>
> **2026-06-04 — Clawpatch report fixes (Codex).** On branch
> `codex/clawpatch-report-fixes`. Addressed Clawpatch findings across server,
> CLI, workflows, and JetBrains: `tellur-server admin create-token` now requires
> an explicit role and records admin-CLI token creation without falsely naming
> the new member as actor; `tellur-core` is a real diagnostic CLI with
> help/version/error behavior; PR provenance workflow uses the checked-out
> binary and immutable PR SHAs from a read-only report job, while PR commenting
> runs in a separate write-scoped job that does not execute PR code; release
> builds use the pinned Rust toolchain; readiness checks verify the migrated
> store; graceful shutdown handles SIGTERM; `tellur init --profile` validates
> supported profiles; `doctor` uses cross-platform executable PATH lookup and
> reports unreadable policy/trace directories; test helpers now require JSON
> problem bodies; read/policy/export/auth/ingest coverage was tightened; SLSA
> materials preserve file identity with percent-encoded file paths; repo ID
> lookup is preferred before repo-name fallback; JetBrains capture now
> deduplicates VFS bursts, re-queues saves that arrive during active capture,
> uses a disposable bounded single-worker queue, logs CLI failures/timeouts, has
> a bounded IDE compatibility range, and includes automated runner tests.
> Verified: `cargo fmt`; `cargo test` (216 Rust tests);
> `cargo clippy --workspace --all-targets -- -D warnings`; `cargo deny check`
> (warnings only, gate green); JetBrains `./gradlew test buildPlugin` with JDK
> 17.
>
> **2026-06-04 — Attribution/SLSA review fixes (Codex).** Addressed 3 P2
> findings on PR #8: ingest now rejects malformed attribution ranges
> (`start_line == 0` or `start > end`) before storage, so SPDX/SLSA line-count
> math can't underflow; and the core SLSA + SPDX structs now serialize the
> **standard JSON field names** (`predicateType`/`buildType`/`configSource`/
> `entryPoint`; `spdxVersion`/`dataLicense`/`SPDXID`/`documentNamespace`/…) via
> `rename_all = "camelCase"` (+ explicit `SPDXID`), so the attestations are
> accepted by SLSA/in-toto and SPDX 2.3 tooling — this also fixes the standalone
> `tellur export slsa|spdx` output. 59 server tests; workspace 204.
>
> **2026-06-04 — Attribution ingest + org SLSA/SPDX export.** On branch
> `feat/server-b6-attribution-slsa`. The hub now ingests line-level attribution
> (`POST /v1/orgs/{org}/repos/{repo}/attributions`, contributor+, per-file
> upsert, schema v7 `attribution` table), which unblocks the deferred compliance
> export: `GET .../repos/{repo}/export/slsa` and `.../export/spdx` (admin) build
> real SLSA v1.0 provenance + SPDX SBOM from the stored attribution via core's
> generators (subject `repo_url`/`commit` are caller-supplied query params).
> Exports run off the async runtime and are audited. Verified: 58 server tests
> (incl. ingest role/tenant + export admin/404) + live smoke (ingest → SLSA with
> materials → SPDX); workspace 203; clippy + cargo-deny green.
>
> **2026-06-04 — Tier 1 B5 (scale & ops, partial) + policy-pull.** On branch
> `feat/server-b5-scale-ops`. Added: a `/metrics` Prometheus endpoint (domain
> counters: ingest/exports/auth-denied/policy-pulls); heavy store ops (org
> report + both exports) now run via `spawn_blocking` so they don't stall the
> async runtime; **packaging** — multi-stage `dist/docker/Dockerfile` +
> `docker-compose.yml` + a CI job that builds the image (docker isn't available
> locally, so it's verified in CI). Follow-up shipped: **`tellur policy pull`**
> in the Apache CLI (small `ureq` client, no TLS deps; validates before writing
> to `.tellur/policies/`) with a live end-to-end test that boots a real hub.
> Verified: 55 server + 26 CLI tests; workspace 200; clippy + cargo-deny green.
> **Deferred (with reasons):** the Postgres backend → its own PR (needs a DB +
> CI service); a persistent/queued job system → lands with Postgres; org-level
> SLSA/SPDX export → needs the hub to ingest line-level attribution first (the
> generators take `FileAttribution`, not events), so it is gated on an
> attribution-ingest feature rather than faking line data. Next: B5-pg
> (Postgres) then B6 (enterprise/SSO).
>
> **2026-06-04 — Tier 1 B4 (central policy & export).** On branch
> `feat/server-b4-policy-export`. **Central policy distribution:**
> `PUT /v1/orgs/{org}/policies/{name}` (admin; body validated as Tellur policy
> YAML via new `PolicyEngine::from_yaml_str`, auto-versioned), `GET .../policies`
> (list), `GET .../policies/{name}` (pull, audited) + `tellur-server admin
> set-policy --file`. **Export portal:** `GET .../export/events` and
> `GET .../export/audit` (admin, rate-limited, audited; audit export includes a
> `chain_intact` integrity flag). New `policy` table (schema v6), storage
> `put_policy/list_policies/get_policy/export_events/export_audit`. Verified: 53
> server tests + live smoke (CLI set-policy → pull, invalid → 400, audit export);
> workspace 197; clippy + deny green. Org-level SLSA/SPDX export and the Apache
> CLI `tellur policy pull` client are noted follow-ups. Next: B5 (scale & ops).
>
> **2026-06-04 — Shared hash-chain helper (refactor).** On branch
> `refactor/server-hash-chain`. Extracted the tamper-evident chain logic that
> review flagged twice (missing head checkpoint) into one
> `crates/server/src/storage/chain.rs`: `read_head`/`write_head` + a generic
> `verify` (walk + prev-linkage + per-row hash recompute closure + head/length
> comparison). The audit log and per-repo event log now both use it, so any
> future chain gets tail-truncation detection for free. Behavior-preserving:
> all 191 tests pass unchanged; clippy + cargo-deny green.
>
> **2026-06-04 — Tier 1 B3 (read & report).** On branch
> `feat/server-b3-read-report`. Added tenant-scoped read endpoints (all audit
> cross-org denials, BOLA-blocked): `GET /v1/orgs/{org}/repos` (repos + event
> counts), `GET /v1/orgs/{org}/repos/{repo}/events` (newest-first, cursor
> pagination by `seq`, limit clamped 1..200, 404 on unknown repo), and
> `GET /v1/orgs/{org}/report` (org rollup: total/distinct-sessions/by-type/
> by-actor/per-repo, audited). Storage: `find_repo`, `list_repos`, `list_events`,
> `org_report`. Verified: 44 server tests (read BOLA/404/401/pagination + report
> aggregation + tenant scoping) + live smoke; workspace fmt/clippy/test (188) +
> deny green. Dashboard wiring (`web/`) deferred to a follow-up. Next: B4
> (central policy & export).
>
> **2026-06-04 — B3 review fixes (Codex).** Addressed 4 P2 findings on PR #4:
> successful event reads are now audited; `Principal` is extracted before
> `Query` so auth/tenant checks (401/403) precede query-param parsing (400);
> corrupt stored payloads surface as an error instead of `null`; and the org
> report is rate-limited + index-backed (`idx_event_org`, schema v5) with a
> job-backed path noted for B5. 47 server tests; workspace 191.
>
> **2026-06-03 — Tier 1 B2 (ingest & verify).** On branch
> `feat/server-b2-ingest`. Added authenticated provenance ingest:
> `POST /v1/orgs/{org}/repos/{repo}/events` (contributor+ role, cross-tenant →
> 403/BOLA). The hub get-or-creates the repo, **recomputes the per-repo hash
> chain** with core's `hash_event` (clients can't forge), and **redacts secrets
> from inbound payloads** before storage (verified live: a secret lands as
> `[REDACTED]` in the DB). Guardrails: 1 MiB body limit (router layer), max 1000
> events/request, and a per-member fixed-window rate limiter → `429`. New
> `event`/`repo` tables (schema v3) + `verify_event_chain`. Verified: 38 server
> tests (incl. ingest BOLA/role/caps/rate-limit + chain tamper) + live smoke;
> workspace fmt/clippy/test (182) + deny green. Next: B3 (read & report).
>
> **2026-06-03 — B2 review fixes (Codex).** Addressed 2 findings on PR #3: the
> per-repo event chain now persists an `event_head` checkpoint (head-hash +
> count) so tail truncation is detected by `verify_event_chain` (P1, mirrors
> `audit_head`); and `docs/THREAT_MODEL.md` is updated for the new ingest trust
> boundary (P2). 39 server tests; 183 workspace.
>
> **2026-06-03 — Tier 1 B1 (identity & tenancy).** On branch
> `feat/server-b1-identity-tenancy`. Added to `crates/server`: an `auth` module
> (viewer/contributor/admin roles; API tokens `tlr_<id>_<secret>` with the secret
> stored only as an **Argon2id** hash; split id/secret for constant-work lookup),
> storage for orgs/members/tokens + a **tamper-evident hash-chained audit log**
> (append/verify/tamper-detect), a deny-by-default axum auth extractor
> (`Principal` from `Authorization: Bearer`), tenant-scoped endpoints
> `GET /v1/me` and `GET /v1/orgs/{org}/me` (object+tenant authz → **BOLA**
> blocked), and a `tellur-server admin create-org/create-token` bootstrap CLI.
> Verified: 25 server tests incl. cross-org BOLA regression; live end-to-end
> smoke (401/200/403); fmt/clippy/test/deny green. Next: B2 (ingest & verify).
>
> **2026-06-03 — B1 review fixes (Codex).** Addressed 4 P2 findings on PR #2:
> audit appends now run in a `BEGIN IMMEDIATE` transaction (atomic across
> connections); the audit chain persists a head-hash/length checkpoint so tail
> truncation is detected by `verify_audit_chain`; Argon2 verification runs off
> the async worker via `spawn_blocking` and releases the DB lock first; and
> presented-but-invalid bearer tokens are now audited (`auth_denied`) while
> header-less requests are not (avoids anonymous audit flooding).
>
> **2026-06-03 — CI hardening.** Fixed a clippy failure that only surfaced on
> the Ubuntu CI (an unnecessary `match` inside a Linux-only `#[cfg]` block that
> macOS-local clippy never compiles). Durable follow-ups: pinned the toolchain in
> `rust-toolchain.toml` (1.96.0) so local/CI clippy match; dropped the redundant
> `cargo-audit` CI step (cargo-deny already scans RustSec); bumped
> `actions/checkout` v4→v5 (Node 20 EOL). CI is green.
>
> **2026-06-03 — Tier 1 B0 shipped (server scaffolding).** New FSL-licensed
> `crates/server` (`tellur-server` binary), the secure foundation before any data
> endpoints. Secure-by-default config (refuses non-loopback bind without explicit
> opt-in; validated at boot), typed errors → RFC 9457 `problem+json` that never
> leak internals on 5xx, a swappable `Store` trait with a SQLite backend
> (WAL/foreign-keys, idempotent migrate), `AppState` + axum router with
> `/healthz` + `/readyz`, structured `tracing`, graceful shutdown. Added
> `SECURITY.md` (coordinated disclosure + CRA reporting), `docs/THREAT_MODEL.md`
> (STRIDE), `deny.toml` + a CI workflow (`cargo fmt`/`clippy`/`test` +
> `cargo-deny` + `cargo-audit`). Verified: 10 server tests; `cargo deny check` →
> advisories/bans/licenses/sources ok; live binary smoke-tested.
>
> **2026-06-03 — Tier 1 implementation plan.** Researched current security &
> compliance standards (OWASP ASVS 5.0, OWASP API Top 10 2023/BOLA, EU CRA,
> SLSA v1.0, NIST SSDF, GDPR, SOC 2/ISO 27001 readiness, Sigstore/SBOM) and
> wrote a secure-by-design, maintainable, scalable plan for the `tellur serve`
> hub: `docs/proposals/TEAM_SERVER_IMPLEMENTATION.md` (new FSL `crates/server`,
> thin-handler/fat-service layering, swappable SQLite→Postgres `Store` trait,
> data-layer tenant scoping to kill BOLA, hash-chained audit log, phased B0–B6
> with CI security gates). Plan only — no code yet.
>
> **2026-06-03 — Team mode Tier 0 shipped.** First build of roadmap item #8.
> `tellur team report` aggregates the `refs/notes/ai` authorship notes of every
> commit in a `--base..--head` range into one team view: AI vs human lines, by
> tool / model / author, plus provenance coverage (which commits carry a note).
> Pure Git-native, no server — notes travel over the existing remote. Pure
> aggregation core in `crates/core/src/report/team_report.rs` (+3 unit tests),
> CLI `tellur team report [--base --head --notes-ref --json]` (+1 integration
> test). Tolerant: missing/unparseable notes count as "without provenance"
> rather than failing. Phase A is complete: an example PR workflow lives at
> `docs/examples/github-actions-team-report.yml` (fetch notes → aggregate →
> upsert one PR comment).
>
> **2026-06-03 — Licensing direction.** Documented the license/structure
> direction in `docs/proposals/LICENSING.md`: Apache-2.0 for the core
> (CLI/core/adapters/schemas/editor); the future team/server component
> (`crates/server`) under FSL-1.1-Apache-2.0 in the same monorepo; contributions
> via DCO, no CLA. Direction only — not implemented; not legal advice.
>
> **2026-06-03 — Team/server mode proposal.** Researched roadmap item #8 against
> the existing local-first primitives (Git notes, per-repo hash chain, daemon,
> export profiles, policy) and the target segments (independent/OSS, SMB,
> corporate). Wrote a phased design proposal at
> `docs/proposals/TEAM_SERVER_MODE.md`: keep local-first as default, use Git
> notes as the zero-infra team transport, and add an optional self-hostable
> `tellur serve` hub. Decided MVP path: **Tier 0 (`tellur team report`, no
> server) first, then Tier 1 (self-hosted hub)**. Proposal only — no code yet;
> reconciled with the PRD (§6 surface 11, §16.2 Layer 5, §32 Step 20; note PRD
> §24 is *Architecture Guardian*, not team mode).
>
> **2026-06-03 — Devin webhook + JetBrains plugin.** Continued roadmap item #7
> ("live capture beyond import") for the remaining adoption tools.
> **Devin** now has a first-class daemon webhook: `POST /webhook/{source}`
> (token-auth, loopback-only) in `crates/core/src/daemon/` normalizes a tool's
> native run/session payload (messages, shell commands, file edits, status) into
> canonical Tellur events, hashing prompt-like fields, redacting commands, and
> **recomputing the hash chain** so provenance cannot be forged. The normalizer
> lives in `crates/core/src/daemon/webhook.rs` (core can't depend on the adapters
> crate). **JetBrains** now has a real IntelliJ Platform plugin under
> `editor/tellur-jetbrains/` (Kotlin/Gradle): it subscribes to `VFS_CHANGES` and
> routes saved/created files to `tellur hooks ingest --source jetbrains
> --auto-init`, capturing AI Assistant and Junie edits live; capture is
> best-effort and off the EDT. The plugin builds outside the Rust workspace CI
> (JDK 17 + IntelliJ SDK via Gradle); the Gradle wrapper is committed and
> `./gradlew buildPlugin` is verified green on JDK 17 (loadable plugin zip).
> Added 4 Rust tests (webhook normalization + authenticated route).
>
> **2026-06-02 — Windsurf live capture.** Started roadmap item #7 ("live capture
> beyond import") for the adoption tools. Windsurf/Cascade now has live capture,
> not just import: `tellur setup windsurf` (and `tellur setup agents`) writes
> Windsurf user settings plus `~/.codeium/windsurf/mcp_config.json`, so the
> VS Code-compatible Tellur extension captures saves with source `windsurf` and
> Windsurf agents can call Tellur MCP tools — mirroring the Cursor integration.
> The same extension capture also records edits made by Continue and Cline/Roo
> Code when they run inside any VS Code-family editor (VS Code, Cursor, Windsurf).
> JetBrains stays import-only (MCP is configured in-IDE, no stable global config);
> Devin stays import-only with live capture available by posting events to the
> authenticated daemon (`POST /events`). Added `windsurf`/`cascade`,
> `jetbrains`/`junie`, `devin`, `continue`, and `cline`/`roo` source
> normalization for `hooks ingest`. Cursor/Windsurf MCP writing is now a shared
> helper. Covered by a new `setup windsurf` CLI test plus extended `setup agents`
> assertions.
>
> **2026-06-01 — Global agent setup.** Added one-time user-level setup for
> Codex, Claude Code, Gemini CLI, Antigravity, Cursor, and VS Code:
> `tellur setup agents` installs global hooks/settings with an absolute `tellur`
> executable path, generates a local Codex personal plugin/marketplace entry for
> manual workflows, writes Gemini CLI hooks, Antigravity hooks/MCP, Cursor
> MCP/settings, and VS Code settings, and routes hook/editor payloads through
> `tellur hooks ingest --auto-init` so new Git repositories can start capturing
> without per-project plugin invocation. Hook ingest now ignores invalid JSON,
> refuses malformed setup config instead of overwriting it, and never falls back
> to whole-tree capture when a tool hook lacks a file path.
>
> **2026-06-02 — Adoption import adapters.** Added five import adapters from the
> roadmap: Windsurf/Cascade, JetBrains AI Assistant / Junie, Devin, Continue, and
> Cline/Roo Code (the latter covers Roo Code via the shared task format). To keep
> these maintainable, the tolerant JSONL/array/envelope parsing loop, payload
> sanitization, prompt hashing, and field extraction now live in one shared
> module `crates/adapters/src/import.rs`; each new adapter only defines its
> tool-specific event-type mapping. `import.rs` also reads single JSON objects,
> envelope objects (`events`/`messages` arrays), and numeric epoch timestamps
> (Cline). `first_string`/`json_path` moved out of the Gemini adapter into the
> shared module. All five are wired into `tellur import <adapter> <source>`,
> registered as built-in adapters, and covered by unit + CLI integration tests.
> They are import-only because none expose a documented local lifecycle hook.
> Hardened after Codex review: envelope wrappers now propagate session/run
> identity onto child events; field extraction is a bounded recursive scan that
> reaches nested objects and Anthropic Messages `content`-block arrays (so Cline/
> Roo `tool_use` writes, Windsurf transcript `code_action`/`user_input`, and
> Continue `nextEditWithHistory`/`fileURI` are recognized); command events
> recover the command from a generic `text` field; and the prompt-hash key set is
> kept in sync with the redaction key set.
>
> **2026-06-02 — README discoverability refresh.** Updated the public README
> with a generated social/hero image, `https://tellur.dev`, clearer value
> proposition, badges, search-friendly AI code provenance language, and a
> stronger star/discovery prompt. No adapter behavior changed.
>
> **2026-06-01 — Adapter hardening.** Tightened adapter imports after review:
> imported events now preserve source IDs, timestamps, session IDs, actors, and
> event types while Tellur recomputes the local hash chain; malformed non-empty
> JSON/JSONL lines now fail instead of being silently dropped; Codex/Copilot
> prompt-like fields are hashed and retained metadata is sanitized; Claude Code
> hook capture is scoped to the hook file path when available; `tellur import
> aider <source>` now uses `<source>` as the Git repository path.
>
> **2026-05-31 — Git notes interop.** Added Git AI-compatible authorship notes
> support under `refs/notes/ai`: `tellur notes export/show/import/fetch/push`
> plus `install-config` for notes fetch/rewrite setup. Notes are a compact
> commit-attestation layer; Tellur's richer event log, redaction state, replay
> data, and policy evidence remain in local/private storage.
>
> **2026-05-31 — Codex + Copilot adapters.** Added `tellur import codex`
> for Codex CLI JSONL event streams/session transcripts and
> `tellur import copilot` for GitHub Copilot metadata exports. Both adapters
> normalize prompts, command events, file writes, and raw metadata into
> Tellur events and are covered by adapter unit tests plus CLI integration
> tests.
>
> **2026-05-31 — Dashboard live data.** The local daemon now backs the session
> replay dashboard with real indexed data: `GET /sessions` returns session rows
> with attribution stats, and `GET /sessions/{id}/events` returns timeline events
> for the selected session. The static dashboard client now understands canonical
> Tellur event wire types (`file.write`, `command.post_execute`, etc.) and
> escapes live event body text before rendering.
>
> **2026-05-31 — Code review & remediation.** A full review found that many
> modules previously marked ✅ were stubs or not wired together (watch, MCP,
> daemon, Claude Code hooks, gc, redact, Homebrew, npm) and that the core
> attribution pipeline was never connected end-to-end. All findings are
> documented in [`docs/FINDINGS.md`](docs/FINDINGS.md) and have been fixed:
> the capture → attribution → index → explain/blame/pr-report pipeline now
> works end-to-end, the daemon is loopback-only + token-authenticated, the MCP
> server speaks real stdio JSON-RPC, hooks use Claude Code's real schema, and
> two security issues (CI command injection, unauthenticated daemon) are
> resolved. That remediation pass was verified with `cargo build`,
> `cargo clippy` (0 warnings), and `cargo test`.

---

## Wat is Tellur

AI code provenance platform. Git vertelt je *wat* er veranderde. Tellur vertelt je *hoe AI participeerde*.

Open-source, lokaal-first, geen cloud dependency. Rust core + CLI, TypeScript editor extension.

---

## PRD Referentie

De PRD bevindt zich op een locatie die Sydney bepaalt. Als je de PRD niet hebt, vraag Sydney.

## Hoe te werken aan deze repo

### Regels voor alle agents

1. **Altijd `PROJECT_STATUS.md` updaten** na elke wijziging — dit is het single source of truth
2. **Build moet groen zijn** voor je commit: `cargo build && cargo test`
3. **Rust code** in `crates/` — editor-integraties in `editor/` (VS Code: TypeScript, JetBrains: Kotlin)
4. **Commits** in het Engels, conventional commits format (`feat:`, `fix:`, `docs:`)
5. **Push altijd** na commit — `git push origin main`
6. **Als je een module afmaakt**, update dan de checklist hieronder met ✅
7. **Als je iets tegenhoudt**, zet het in de Blockers sectie
8. **Geen breaking changes** aan bestaande schemas zonder Sydney's goedkeuring

### Build & Test

```bash
cargo build
cargo clippy --workspace --all-targets -- -D warnings
cargo test
cargo deny check          # supply-chain gate (licenses + advisories + sources)
cd editor/tellur-vscode
npm run compile
npm run test:unit
npm run test:extension
# JetBrains plugin (Kotlin/Gradle, niet via cargo/CI gebouwd):
cd ../tellur-jetbrains && ./gradlew buildPlugin
```

### Structuur

```
Tellur/
├── PROJECT_STATUS.md        ← DIT BESTAND
├── Cargo.toml               ← Rust workspace root
├── crates/
│   ├── core/                ← Core library (schema, storage, attribution, policy,
│   │                          redaction, export, reports, notes, remap, daemon, mcp)
│   ├── cli/                 ← CLI binary (tellur command)
│   ├── adapters/            ← AI tool adapters (import + hook/payload normalization)
│   └── server/              ← Tier 1 team hub (tellur-server, FSL-1.1-ALv2)
├── schemas/                 ← JSON Schema definities
├── web/                     ← Static session-replay dashboard
├── .github/workflows/       ← GitHub Actions
└── editor/
    ├── tellur-vscode/       ← VS Code / Cursor / Windsurf extension (TypeScript)
    └── tellur-jetbrains/    ← JetBrains IDE plugin (Kotlin/Gradle)
```

> Authoritative architecture map: zie [`AGENTS.md`](AGENTS.md) ("Start Here" + Architecture Map).

---

## PRD Implementatie Checklist

### Phase 1: Foundation (PRD secties 1-7)

| # | Module | PRD Sectie | Status | Details |
|---|--------|-----------|--------|---------|
| 1 | Project scaffold | 32.1 | ✅ Done | Rust workspace, 3 crates |
| 2 | Core schemas (Session, Event, Attribution, Policy, ProvenanceBundle, PRReport) | 4-6 | ✅ Done | `crates/core/src/schema/types.rs` |
| 3 | ID generation (hash_event, hash_content, generate_session_id, etc.) | 4.2 | ✅ Done | `crates/core/src/schema/ids.rs` |
| 4 | JSON Schema definities | 4-6 | ✅ Done | `schemas/*.json` |
| 5 | EventWriter (JSONL + SHA-256 hash chain) | 7 | ✅ Done | `crates/core/src/storage/event_log.rs` |
| 6 | TraceIndex (SQLite) | 7.3 | ✅ Done | `crates/core/src/storage/index.rs` |
| 7 | RepoStorage (.tellur directory) | 7.1 | ✅ Done | `crates/core/src/storage/repo.rs` |
| 8 | File change capture (git diff + blob SHA) | 8.3 | ✅ Done | `crates/core/src/storage/file_watcher.rs` |

### Phase 2: Core Intelligence (PRD secties 8-14)

| # | Module | PRD Sectie | Status | Details |
|---|--------|-----------|--------|---------|
| 9 | AttributionEngine (line-level patch attribution) | 9 | ✅ Done | `crates/core/src/attribution/engine.rs` |
| 10 | RedactionEngine (regex secret detection) | 14 | ✅ Done | `crates/core/src/redaction/mod.rs` |
| 11 | PolicyEngine (YAML rules, sensitive paths) | 13 | ✅ Done | `crates/core/src/policy/mod.rs` |
| 12 | AgentAdapter trait (async_trait) | 8.3 | ✅ Done | `crates/core/src/adapter/mod.rs` |
| 13 | Built-in adapters (Claude Code, Aider, Cursor, Generic, Codex, Copilot, Gemini CLI, Antigravity) | 8.3 | ✅ Done | `crates/core/src/adapter/builtin.rs` + `crates/adapters/src/*` |
| 14 | Claude Code adapter implementation | 8.1 | ✅ Done | Real Claude Code hook schema (PostToolUse/SessionStart), `tellur hooks install`, stdin payload handler `tellur hooks claude` wired to capture pipeline and scoped to hook file path when available, transcript parse |
| 15 | Aider adapter implementation | 8.2 | ✅ Done | Git log parser, Aider pattern detection, source repo path honored by CLI import |
| 16 | Cursor adapter implementation | 8.2 | ✅ Done | Cursor MCP/settings setup, VS Code-compatible extension capture, JSON/JSONL trace parsing, workspace detection, adapter tests |
| 16a | Codex CLI adapter implementation | 8.2 | ✅ Done | JSONL event stream/session transcript import via `tellur import codex <file>`, command/prompt/file-write normalization, prompt hashing, strict JSONL errors |
| 16b | GitHub Copilot adapter implementation | 8.2 | ✅ Done | Metadata JSON/JSONL import via `tellur import copilot <file>`, accepted suggestion + prompt metadata normalization, prompt hashing, no raw metadata payload |
| 16c | Unified agent/editor/repo setup | 8.1/8.3/10/23 | ✅ Done | Tellur installs once per machine. The release installers deploy the CLI + VSIX + JetBrains ZIP and launch `tellur setup`; the wizard globally configures Codex, Claude Code, Gemini CLI, Antigravity, Cursor, Windsurf, VS Code, JetBrains and optional Team Hub login. Repositories activate on first agent/editor activity, including idempotent Git hooks and notes config; `.tellur/disable` opts a repo out. `setup update/status` reconcile and verify the installation. |
| 16d | Gemini CLI adapter implementation | 8.2 | ✅ Done | `tellur setup gemini-cli`, `~/.gemini/settings.json` hooks, JSONL import via `tellur import gemini-cli <file>`, prompt hashing and metadata sanitization |
| 16e | Antigravity 2.0 adapter implementation | 8.2/23 | ✅ Done | `tellur setup antigravity`, `~/.gemini/config/hooks.json`, Antigravity app/CLI MCP configs, JSONL import via `tellur import antigravity <file>` |
| 16f | Shared import loop | 8.2 | ✅ Done | `crates/adapters/src/import.rs`: tolerant JSONL/array/envelope/single-object reader, line-specific errors, sanitized+prompt-hashed payloads, nested-field extraction, numeric epoch timestamps. Reused by the adoption adapters below |
| 16g | Windsurf/Cascade adapter | 8.2 | ✅ Done | Import via `tellur import windsurf <file>`; Cascade tool calls, file edits, commands, chat turns |
| 16h | JetBrains AI Assistant / Junie adapter | 8.2 | ✅ Done | Import via `tellur import jetbrains <file>`; AI Assistant + Junie action logs (JSON/array/JSONL) |
| 16i | Devin adapter | 8.2 | ✅ Done | Import via `tellur import devin <file>`; run/session export (object/array/JSONL) for cloud-agent provenance |
| 16j | Continue adapter | 8.2 | ✅ Done | Import via `tellur import continue <file>`; `dev_data` JSONL with nested `data` payloads |
| 16k | Cline / Roo Code adapter | 8.2 | ✅ Done | Import via `tellur import cline <file>`; `ui_messages.json`/`api_conversation_history.json` task history, shared by Roo Code |
| 16l | Windsurf live capture | 8.1/10/23 | ✅ Done | `tellur setup windsurf` writes Windsurf user settings + `~/.codeium/windsurf/mcp_config.json`; VS Code-compatible extension captures saves with source `windsurf` (mirrors Cursor). Same extension also covers Continue/Cline/Roo running in a VS Code-family editor. `hooks ingest` source normalization for windsurf/jetbrains/devin/continue/cline |
| 16m | Devin webhook live capture | 22/23 | ✅ Done | Daemon `POST /webhook/{source}` (token-auth, loopback-only) normalizes native run/session payloads → events with recomputed hash chain (`crates/core/src/daemon/webhook.rs`). 4 tests |
| 16n | JetBrains live-capture plugin | 10 | ✅ Done | `editor/tellur-jetbrains/` IntelliJ Platform plugin (Kotlin/Gradle): `VFS_CHANGES` listener → `hooks ingest --source jetbrains --auto-init`, settings UI for the tellur path. Builds outside Rust CI (JDK 17 + IntelliJ SDK) |

### Phase 3: CLI (PRD sectie 8.1)

| # | Module | PRD Sectie | Status | Details |
|---|--------|-----------|--------|---------|
| 17 | `tellur init` | 8.1 | ✅ Done | CLI main.rs |
| 18 | `tellur doctor` | 8.1 | ✅ Done | Detecteert AI tools |
| 19 | `tellur status` | 8.1 | ✅ Done | Sessions overview |
| 20 | `tellur explain <file:line>` | 8.1 | ✅ Done | Line attribution lookup |
| 21 | `tellur blame <file>` | 8.1 | ✅ Done | File-wide attribution |
| 22 | `tellur pr-report` | 12 | ✅ Done | Risk report + markdown |
| 23 | `tellur policy check` | 13 | ✅ Done | Policy evaluation |
| 24 | `tellur watch` | 8.1 | ✅ Done | Real `notify` filesystem watcher with debounce → git-diff capture → attribution → index (incl. untracked/new files) |
| 25 | `tellur event` | 8.1 | ✅ Done | Single event emission |
| 26 | `tellur export` | 15 | ✅ Done | Provenance bundle export |
| 27 | `tellur import` | 8.1 | ✅ Done | JSONL import |
| 28 | `tellur verify` | 11 | ✅ Done | Hash chain verification |
| 29 | `tellur sessions` | 8.1 | ✅ Done | Session listing |
| 30 | `tellur gc` | 8.1 | ✅ Done | Real retention-based deletion (keep_days from config), rewrites logs + rebuilds index; `--dry-run` is truly dry |
| 31 | `tellur redact` | 14 | ✅ Done | Rewrites stored payloads in place, records RedactionInfo, re-seals hash chain so `verify` stays intact |
| 31a | `tellur team report` | §6.11/§32 Step 20 | ✅ Done | Tier 0 team mode: aggregates `refs/notes/ai` notes over a `--base..--head` range into AI involvement by tool/model/author + provenance coverage. Markdown/`--json`. No server. `crates/core/src/report/team_report.rs` |

### Phase 4: Reports & Export (PRD secties 12, 15, 20)

| # | Module | PRD Sectie | Status | Details |
|---|--------|-----------|--------|---------|
| 32 | PRReportGenerator | 12 | ✅ Done | `crates/core/src/report/pr_report.rs` |
| 33 | Markdown report output | 12.3 | ✅ Done | `PRReportGenerator::to_markdown()` |
| 34 | Provenance export (6 profiles) | 15, 20 | ✅ Done | `crates/core/src/storage/export.rs` |
| 35 | GitHub Action (PR check) | 12.4 | ✅ Done | `.github/workflows/tellur.yml` |

### Phase 5: Editor Extension (PRD sectie 10)

| # | Module | PRD Sectie | Status | Details |
|---|--------|-----------|--------|---------|
| 36 | VS Code/Cursor-compatible extension | 10 | ✅ Done | Full extension: client, decorations, tree views, commands, auto-init, auto-watch, save capture through `hooks ingest` |
| 36a | JetBrains IDE plugin | 10 | ✅ Done | `editor/tellur-jetbrains/` — IntelliJ Platform plugin, `VFS_CHANGES` listener → `hooks ingest --source jetbrains`, settings UI. Live capture for AI Assistant/Junie. Builds outside Rust CI |
| 37 | Inline attribution decorations | 10.1 | ✅ Done | Purple (AI) vs green (human) line decorations | |
| 38 | Hover cards (origin, model, confidence) | 10.2 | ✅ Done | Explain command shows origin, model, confidence, session | |
| 39 | Sidebar panel | 10.3 | ✅ Done | Sessions + Attributions tree views in activity bar | |
| 40 | Session explorer | 10.4 | ✅ Done | SessionProvider tree with agent, model, event count | |

### Phase 6: Advanced Features (PRD secties 16-25)

| # | Module | PRD Sectie | Status | Details |
|---|--------|-----------|--------|---------|
| 41 | Session replay web UI | 16 | ✅ Done | Dark theme timeline, session sidebar, attribution bar, diff viewer, demo fallback, live daemon data via `/sessions` + `/sessions/{id}/events` | Web dashboard |
| 42 | Git remapping | 17 | ✅ Done | SHA remap via git diff-tree, rebase detection, 3 tests | |
| 43 | SLSA/SPDX export | 20 | ✅ Done | SLSA v1.0 provenance + SPDX 2.3 SBOM with AI metadata, 2 tests | |
| 44 | HTTP daemon (axum) | 22 | ✅ Done | `tellur daemon` (loopback-only, token-auth, Host check). Server **recomputes the hash chain** via EventWriter — clients cannot forge provenance. 7 endpoints incl. `POST /webhook/{source}` for cloud-agent (Devin) live capture. |
| 45 | MCP server | 23 | ✅ Done | `tellur mcp` — real stdio JSON-RPC 2.0 (initialize/tools/list/tools/call). 6 tools backed by actual index/policy/verify queries. |
| 46 | Team/server mode | §6.11 / §16.2 L5 / §32 Step 20 | ✅ In preview | Tier 0 and the self-host hub include ingest/read/report/policy/export, SQLite/Postgres, per-repo RBAC, OIDC SSO/device login, SCIM users + role groups, durable jobs, source proxy, GitHub App integration, and the embedded dashboard. Production deployment docs and hosted operations remain open. |
| 47 | Plugin/adapter SDK | §8.3 / §32 Step 18 | ❌ Not started | Requires stable adapter/event API |

### Phase 7: Distribution (PRD sectie 32.3)

| # | Module | PRD Sectie | Status | Details |
|---|--------|-----------|--------|---------|
| 48 | Cross-compilation | 32.3 | ✅ Done | macOS arm64/x64, Linux x64 musl + arm64 GNU, Windows x64; canonical archives + SHA-256 sidecars. Linux release targets compile in PR CI before tag publication. |
| 49 | Homebrew formula | 32.3 | ⚠️ Template only | `dist/tellur.rb` still has a placeholder checksum and is not a usable published channel; the GitHub installer is supported. |
| 50 | npm package (CLI wrapper) | 32.3 | ⚠️ Implemented, unpublished | JS wrapper + SHA-256-verifying downloader; unsupported until the package and release assets are published. |
| 51 | GitHub Release automation | 32.3 | ✅ Done | Five CLI targets + checksums, version-matched VSIX, JetBrains ZIP, Unix/Windows installers. CI builds editor packages and Linux release binaries and exercises both installers E2E before a `v*` tag publishes the release. |

---

## Wat is NIET uit de PRD opgepakt

Deze onderdelen staan in de PRD maar zijn bewust overgeslagen of vereisen Sydney's beslissing:

1. **Pricing / Business model** (PRD sectie 27-31) — niet relevant voor dev, Sydney beslist
2. **Enterprise team/server follow-ups** (PRD §6.11 / §16.2 Layer 5 / §32 Step 20)
   — Tier 0 and the self-host hub preview are implemented on both the SQLite and
   Postgres backends, with enterprise RBAC/SSO/SCIM, durable jobs, and
   SCIM Group-based role sync. Remaining work is production deployment guidance
   and verification against a real GitHub App installation.
3. **SOC 2 compliance** (PRD sectie 26) — far future
4. **Plugin SDK** (PRD sectie 25) — API stabiliteit eerst nodig
5. **Release signing** (PRD sectie 20) — na v1.0 (SLSA/SPDX *export* is wel klaar)
6. ~~**Session replay web dashboard met live data**~~ — ✅ Done via local daemon endpoints
7. ~~**GitHub Copilot / Codex CLI adapters**~~ — ✅ Done as import adapters

---

## Huidige Test Status

```
360 Rust tests, 0 failures, 0 clippy warnings; `cargo deny check` green.
(verified 2026-08-04 via `cargo test`; Postgres tests no-op without `TELLUR_TEST_DATABASE_URL`)
- core:      78  (76 library + 2 CLI-contract integration: schema/event round-trip, glob, storage, hash-chain verify+reseal,
             index session/attribution round-trip, capture pipeline, attribution,
             redaction, policy, export, PR report, team report, daemon/webhook)
- adapters:  56  (Claude Code, Aider, Cursor, Codex, Copilot, Gemini CLI,
             Antigravity, Windsurf, JetBrains, Devin, Continue, Cline/Roo, Generic,
             + shared import loop)
- cli:       60  (20 unit incl. setup URL validation, Codex patch-path parsing,
             push high-water-mark/tombstones, prompt-excerpt redaction,
             host/segment encoding; 40 integration incl. unified setup/package
             status, exact/claimed notes, single-owner setup, live hook capture,
             policy pull, and policy-check enforcement)
- server:    166 (59 lib unit + 107 integration: auth/BOLA, ingest, repo RBAC,
             OIDC SSO + device login, SCIM users/groups, jobs, dashboard reads,
             attribution + tombstones, A12 source/blob proxy, dashboard routes,
             Postgres surface — Postgres tests run only with TELLUR_TEST_DATABASE_URL)
- dashboard: 63 vitest (pure helpers: router, source/buildTemplates, timeline,
             commands, i18n catalog parity) + 5 Playwright E2E (real built bundle
             in Chromium with the /v1 API mocked — frontend E2E, not full-stack)
- editor:    VS Code — TS compile + 5 unit + extension tests. JetBrains —
             `./gradlew buildPlugin` on JDK 17 (built outside the Rust CI).
```

Run: `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test`
(set `TELLUR_TEST_DATABASE_URL` to include the Postgres backend tests). Dashboard
(from `crates/server/ui`, **one script per command** — `pnpm run` treats extra
words as args, not multiple scripts): `pnpm install`, then `pnpm check`,
`pnpm test`, `pnpm build`, `pnpm e2e`. VS Code:
`cd editor/tellur-vscode && npm run compile && npm run test:unit && npm run test:extension`.

---

## Blockers

| Blocker | Impact | Oplossing |
|---------|--------|-----------|
| Geen | — | — |

---

## Tech Stack Beslissingen

| Keuze | PRD Sectie | Reden |
|-------|-----------|-------|
| Rust (core + CLI) | 32.1 | Performance, safety, single binary |
| SQLite (index) | 7.3 | Local, zero-config, fast queries |
| JSONL (event log) | 7.2 | Append-only, human-readable, git-friendly |
| SHA-256 (hash chain) | 7.2 | Tamper evidence, lightweight |
| TypeScript (editor) | 10 | VS Code API vereist TS |
| YAML (policy) | 13 | Git-friendly config |

---

## Git Log (laatste 5 commits)

```
5c9af99 Merge #40 fix(oidc): opt-in for http issuers on trusted networks + clear startup error
57e27be Merge #39 feat(timeline): opt-in prompt excerpts + dynamic session timeline
ad1131c Merge #38 fix(dashboard): serve app shell for file-provenance deep links (not 404)
79d8dd7 Merge #32 docs(readme): revise for accuracy and readability
(earlier: #33 tellur login+push, #34 device-login token, #35 source connection + blob proxy,
 #36 GitHub App proposal, #37 push line-level attribution)
```

---

## Volgende Stappen (prioriteit)

1. ~~**Claude Code hook installer**~~ — ✅ Done
2. ~~**Aider commit parser**~~ — ✅ Done  
3. ~~**VS Code/Cursor-compatible extension**~~ — ✅ Done
4. ~~**Codex CLI adapter**~~ — ✅ Done
5. ~~**GitHub Copilot adapter**~~ — ✅ Done
6. ~~**Next first-party adapters for adoption**~~ — ✅ Done as import adapters (Windsurf/Cascade, JetBrains AI / Junie, Devin, Continue, Cline/Roo Code)
7. **Live capture beyond import** — ✅ Done for all adoption tools' durable
   surfaces. Windsurf/Cascade live via `tellur setup windsurf` (VS Code-compatible
   extension + MCP); Continue and Cline/Roo covered by the same extension inside
   any VS Code-family editor; JetBrains live via the `editor/tellur-jetbrains`
   IntelliJ plugin (`VFS_CHANGES` → `hooks ingest`); Devin live via the daemon
   `POST /webhook/devin` endpoint. Remaining (optional): lifecycle-hook capture
   for Windsurf/JetBrains if/when they document a local hook API. Marketplace
   publication is optional discovery work; direct release packages are built.
8. **Team/server mode** — proposal
   [`docs/proposals/TEAM_SERVER_MODE.md`](docs/proposals/TEAM_SERVER_MODE.md);
   reconciled with PRD §6.11 / §16.2 Layer 5 / §32 Step 20.
   **Tier 0 ✅ done** (`tellur team report` + example PR CI).
   **Tier 1 implementation complete/in preview** per
   [`docs/proposals/TEAM_SERVER_IMPLEMENTATION.md`](docs/proposals/TEAM_SERVER_IMPLEMENTATION.md)
   (B0–B6). **B0 ✅** scaffolding (config, errors, Store+SQLite, health, SECURITY/
   STRIDE/cargo-deny/CI). **B1 ✅** (branch `feat/server-b1-identity-tenancy`):
   roles + Argon2id API tokens, orgs/members, hash-chained audit log, deny-by-
   default auth extractor, tenant-scoped `/v1/me` + `/v1/orgs/{org}/me` (BOLA
   blocked), admin bootstrap CLI. **B2 ✅** (branch `feat/server-b2-ingest`):
   authenticated `POST .../repos/{repo}/events` with server-recomputed per-repo
   hash chain, inbound secret redaction, body/size + rate-limit guardrails.
   **B3 ✅** tenant-scoped read endpoints (`GET .../repos`, paginated
   `.../events`, `.../report`). **B4 ✅** (branch `feat/server-b4-policy-export`):
   central policy distribution (`PUT/GET .../policies[/{name}]`, validated +
   versioned) and an export portal (`GET .../export/events|audit`, admin,
   rate-limited). **B5 (partial) ✅** (branch `feat/server-b5-scale-ops`):
   `/metrics`, heavy-op offload, Docker/Compose packaging + CI build, and the
   `tellur policy pull` client. **Attribution ingest + SLSA/SPDX export ✅**
   (branch `feat/server-b6-attribution-slsa`): per-repo SLSA v1.0 + SPDX from
   ingested line-level attribution. **Postgres backend ✅** (branch
   `feat/server-postgres`): `PostgresStore` (r2d2 pool, NoTls) behind the same
   `Store` trait, selected via `TELLUR_DATABASE_URL`, with a `postgres:16` CI
   service running the integration tests. **B6 (enterprise) ✅ complete:**
   **fine-grained per-repo RBAC** (branch `feat/server-b6-repo-rbac`) — additive
   per-repo role grants (`max(org_role, grant)`); **OIDC SSO**
   (branch `feat/server-b6-oidc-sso`) — code+PKCE browser login, opaque session
   cookies, email-provisioned members; **SCIM 2.0** (branch `feat/server-b6-scim`)
   — `/scim/v2/Users` provisioning/deprovisioning via an org-scoped token, with
   `member.active` gating all auth paths. Still open: **queued/durable jobs**;
   SCIM Group-based role sync.
9. **Plugin SDK** — requires stable adapter/event API

---

## Concurrentieonderzoek (2026-05-31)

Directe concurrenten die hetzelfde probleem oplossen:

### 1. Git AI (usegitai.com)
- **Open-source** git extension
- Gebruikt **Git Notes** voor AI authorship tracking
- `git-ai blame` + `git-ai stats` commands
- Ondersteunt Agent Hooks voor automatische tracking
- `refs/notes/ai` open standard
- **+/−**: Git-native (goed), maar geen policy engine, geen redaction, geen export profiles

### 2. AgentBlame (mesa.dev/agentblame)
- **Line-level AI attribution** via git notes
- Combineert git blame met AI attributie
- Tool/model breakdown dashboards
- Ondersteunt Cursor, Claude Code, OpenCode
- **+/−**: Mooie UX, maar gesloten platform (SaaS?), geen policy/redaction

### 3. Entire CLI
- Captureert AI session transcripts in git commits
- Line-level AI vs human attributie
- **+/−**: Focus op session capture, minder breed dan Tellur

### 4. AI Footprint
- Git-native AI code tracking
- **+/−**: Vroeg stadium, vergelijkbare aanpak

### Tellur differentiators
1. **Policy engine** — YAML-based rules voor sensitive paths, required reviews, test evidence
2. **Secret redaction** — regex-based detection van API keys, tokens, passwords
3. **6 export profiles** — developer, OSS, corporate, audit, release, CI
4. **PR risk reports** — risico scoring, AI involvement stats, reviewer checklist
5. **Tamper-evident log** — SHA-256 hash chain (geen concurrent heeft dit)
6. **Multi-adapter** — Claude Code, Aider, Cursor, Codex, Copilot, Gemini CLI, Antigravity, Generic uit de doos
7. **Rust** — single binary, snel, geen runtime dependency
8. **SLSA/SPDX ready** — export naar supply chain formats

### Actiepunten uit concurrentie
- Git Notes integratie uitbreiden met release/CI workflows waar nuttig
- Dashboard metrics (tool/model breakdown) toevoegen aan `tellur stats`
- `/ask` feature (chat met AI die code schreef) — uniek, overwegen voor later

---

*Agents: update dit bestand na elke werk sessie. Voeg je naam + timestamp toe aan de "Last updated" regel.*
