//! Command-line argument definitions (clap).
//!
//! This module holds only the declarative CLI surface — the `Cli` root plus the
//! command/subcommand enums. The dispatch in `main` maps each variant to a
//! command implementation in the relevant `commands`-style module.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tellur")]
#[command(
    version,
    about = "AI Code Provenance — line-level attribution, session replay, PR risk reports"
)]
#[command(
    long_about = "Tellur records, attributes, and reports on AI-assisted development.\n\n\
Git tells you what changed. Tellur tells you how AI participated."
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Initialize Tellur in the current repository
    Init {
        /// Setup profile: default | team | oss-maintainer
        #[arg(long, default_value = "default")]
        profile: String,
    },

    /// Check Tellur setup and detect AI tools
    Doctor,

    /// Show current Tellur status
    Status,

    /// Explain who/what changed a specific line
    Explain {
        /// File path and line number (e.g., src/main.rs:42)
        target: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show AI attribution for a file
    Blame {
        /// File path
        file: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate a PR risk report
    PrReport {
        /// Base ref (default: main)
        #[arg(long, default_value = "main")]
        base: String,
        /// Head ref (default: HEAD)
        #[arg(long, default_value = "HEAD")]
        head: String,
    },

    /// Check policy compliance
    Policy {
        #[command(subcommand)]
        action: PolicyActions,
    },

    /// Export provenance data
    Export {
        /// Export format: native | agent-trace | markdown | json
        #[arg(long, default_value = "native")]
        format: String,
        /// Output file (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import events from an external source
    Import {
        /// Adapter to import from: claude-code | aider | cursor | generic | codex | copilot | gemini-cli | antigravity | windsurf | jetbrains | devin | continue | cline
        adapter: String,
        /// Source path
        source: PathBuf,
    },

    /// Start watching for AI development activity
    Watch {
        /// Agent/tool identifier to attach to inferred file changes
        #[arg(long, default_value = "watch")]
        agent_id: String,
        /// Human-readable agent/tool name for the session list
        #[arg(long, default_value = "Tellur Watch")]
        agent_name: String,
        /// Optional model identifier, for example openai:gpt-5 or copilot:gpt-4.1
        #[arg(long)]
        model_id: Option<String>,
    },

    /// Emit a single event (for generic adapter / CI)
    Event {
        /// Event type (e.g., file.write, command.post_execute)
        #[arg(long)]
        event_type: String,
        /// Session ID
        #[arg(long)]
        session: String,
        /// File path (for file events)
        #[arg(long)]
        file: Option<String>,
        /// Command (for command events)
        #[arg(long)]
        command: Option<String>,
        /// Exit code (for command events)
        #[arg(long)]
        exit_code: Option<i32>,
        /// Structured JSON payload to merge into the event payload
        #[arg(long)]
        payload_json: Option<String>,
    },

    /// Garbage collect expired data
    Gc {
        /// Dry run — show what would be deleted
        #[arg(long)]
        dry_run: bool,
    },

    /// Verify provenance integrity
    Verify,

    /// Redact sensitive content from stored events
    Redact,

    /// Show session details
    Sessions {
        /// Specific session ID to show
        session_id: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run the local HTTP daemon (event ingestion API)
    Daemon {
        /// Host to bind (loopback only)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind
        #[arg(long, default_value_t = 4917)]
        port: u16,
    },

    /// Run the MCP server over stdio (for AI agents)
    Mcp,

    /// Manage Git AI-compatible authorship notes (default ref: refs/notes/ai)
    Notes {
        #[command(subcommand)]
        action: NotesActions,
    },

    /// Team-level reports aggregated from Git AI authorship notes (no server)
    Team {
        #[command(subcommand)]
        action: TeamActions,
    },

    /// Manage editor/agent hook integrations
    Hooks {
        #[command(subcommand)]
        action: HookActions,
    },

    /// Set up Tellur for this machine, repository, and optional Team Hub
    Setup {
        #[command(subcommand)]
        action: Option<SetupActions>,
        /// Team Hub URL; omit to choose interactively or stay local-only
        #[arg(long, conflicts_with = "local_only")]
        hub: Option<String>,
        /// Configure local capture without connecting a Team Hub
        #[arg(long)]
        local_only: bool,
        /// Git remote used for provenance notes
        #[arg(long, default_value = "origin")]
        remote: String,
        /// Do not install the background Team Hub sync service
        #[arg(long)]
        no_background: bool,
        /// Do not open a browser during Team Hub login
        #[arg(long)]
        no_browser: bool,
        /// Accept safe defaults and do not prompt (for automation)
        #[arg(long)]
        yes: bool,
    },

    /// One-time zero-touch setup: hub login, agent capture, and auto-push git hooks
    Connect {
        /// Hub base URL (or env TELLUR_HUB_URL), e.g. https://hub.example.com
        #[arg(long)]
        hub: Option<String>,
        /// Remote to sync authorship notes with (fetch + auto-push)
        #[arg(long, default_value = "origin")]
        remote: String,
        /// Skip the hub device login step
        #[arg(long)]
        no_login: bool,
        /// Skip installing global editor/agent capture integrations
        #[arg(long)]
        no_agents: bool,
        /// Also install an always-on per-user background service that pushes to
        /// the hub on an interval (launchd on macOS, systemd --user on Linux)
        #[arg(long)]
        background: bool,
        /// Interval in seconds for the background push service (with --background)
        #[arg(long, default_value_t = 900)]
        push_interval: u64,
        /// Do not open a browser during the login step
        #[arg(long)]
        no_browser: bool,
        /// Show what `tellur connect` has installed in this repository
        #[arg(long)]
        status: bool,
        /// Remove the git hooks and notes config installed by `tellur connect`
        #[arg(long)]
        remove: bool,
    },

    /// Sign in to a Tellur team hub (opens a browser; no token to copy/paste)
    Login {
        /// Hub base URL (or env TELLUR_HUB_URL), e.g. https://hub.example.com
        #[arg(long)]
        hub: Option<String>,
        /// Do not try to open a browser; just print the URL and code
        #[arg(long)]
        no_browser: bool,
    },

    /// Remove stored credentials for a hub
    Logout {
        /// Hub base URL (or env TELLUR_HUB_URL). Defaults to the only saved hub.
        #[arg(long)]
        hub: Option<String>,
    },

    /// Push locally-captured events to a team hub (incremental + idempotent)
    Push {
        /// Hub base URL (or env TELLUR_HUB_URL). Defaults to the only saved hub.
        #[arg(long)]
        hub: Option<String>,
        /// Organization id (or stored from `tellur login`, or env TELLUR_HUB_ORG)
        #[arg(long)]
        org: Option<String>,
        /// Repository name on the hub (default: this repo's directory name)
        #[arg(long)]
        repo: Option<String>,
        /// Bearer token (or stored credentials, or env TELLUR_HUB_TOKEN)
        #[arg(long)]
        token: Option<String>,
        /// Show what would be pushed without sending anything
        #[arg(long)]
        dry_run: bool,
        /// Ignore the saved high-water mark and re-push every local event
        #[arg(long)]
        reset: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum NotesActions {
    /// Export indexed attribution to a Git AI-compatible note
    Export {
        /// Commit to annotate
        #[arg(default_value = "HEAD")]
        commit: String,
        /// Notes ref to write
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
        /// Print note content instead of writing Git notes
        #[arg(long)]
        print: bool,
    },
    /// Explicitly attest that a missed commit's added lines were AI-authored
    AttestAi {
        /// Commit to annotate
        #[arg(default_value = "HEAD")]
        commit: String,
        /// AI session that produced the commit
        #[arg(long)]
        session: String,
        /// Agent/tool identifier
        #[arg(long)]
        agent: String,
        /// Model identifier when known
        #[arg(long, default_value = "unknown")]
        model: String,
        /// Notes ref to write
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
        /// Replace an existing authorship note
        #[arg(long)]
        force: bool,
    },
    /// Show and parse the authorship note for a commit
    Show {
        /// Commit to inspect
        #[arg(default_value = "HEAD")]
        commit: String,
        /// Notes ref to read
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
        /// Output parsed note as JSON
        #[arg(long)]
        json: bool,
    },
    /// Import a Git AI authorship note into the local Tellur index
    Import {
        /// Commit to import notes from
        #[arg(default_value = "HEAD")]
        commit: String,
        /// Notes ref to read
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
    },
    /// Fetch authorship notes from a remote
    Fetch {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
        /// Notes ref to fetch
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
    },
    /// Push authorship notes to a remote
    Push {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
        /// Notes ref to push
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
    },
    /// Configure this repository to fetch and rewrite authorship notes
    InstallConfig {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
        /// Notes ref to configure
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum TeamActions {
    /// Aggregate AI involvement across a commit range from Git authorship notes
    Report {
        /// Base ref (default: main)
        #[arg(long, default_value = "main")]
        base: String,
        /// Head ref (default: HEAD)
        #[arg(long, default_value = "HEAD")]
        head: String,
        /// Notes ref to read
        #[arg(long, default_value = tellur_core::notes::GIT_AI_NOTES_REF)]
        notes_ref: String,
        /// Output the report as JSON instead of Markdown
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum HookActions {
    /// Install Tellur hooks into Claude Code settings (.claude/settings.json)
    Install {
        /// Which tool's hooks to install
        #[arg(default_value = "claude-code")]
        tool: String,
    },
    /// Internal: handle a Claude Code hook payload from stdin
    #[command(hide = true)]
    Claude,
    /// Internal: ingest a supported agent hook payload from stdin
    #[command(hide = true)]
    Ingest {
        /// Hook source: claude-code | codex | gemini-cli | antigravity | vscode | cursor
        #[arg(long)]
        source: String,
        /// Initialize Tellur automatically when inside a Git repository
        #[arg(long)]
        auto_init: bool,
        /// Print an empty JSON object for hook systems that require JSON stdout
        #[arg(long, hide = true)]
        json_response: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum SetupActions {
    /// Reconcile integrations and automation after upgrading Tellur
    Update,
    /// Install global Codex, Claude Code, Cursor, and VS Code integrations
    Agents {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Install global Codex integration
    Codex {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Install global Claude Code integration
    ClaudeCode {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Install global Cursor integration
    Cursor {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Install global VS Code integration
    Vscode {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Install global Windsurf / Cascade integration
    Windsurf {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Install global Gemini CLI integration
    GeminiCli {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Install global Google Antigravity 2.0 integration
    Antigravity {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Show global integration and current-repository status
    Status {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
        /// Git remote used for provenance notes
        #[arg(long, default_value = "origin")]
        remote: String,
    },
    /// Remove global integrations installed by Tellur
    Uninstall {
        /// Override home directory, intended for tests and portable installs
        #[arg(long)]
        home: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum PolicyActions {
    /// Check all policies
    Check {
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Explain what a policy does
    Explain {
        /// Policy rule ID
        rule_id: Option<String>,
    },
    /// Pull a central policy from a Tellur team hub into this repo
    Pull {
        /// Organization id on the hub
        #[arg(long)]
        org: String,
        /// Policy name to fetch
        #[arg(long, default_value = "default")]
        name: String,
        /// Hub base URL (or env TELLUR_HUB_URL), e.g. http://127.0.0.1:4920
        #[arg(long)]
        hub: Option<String>,
        /// Bearer token (or env TELLUR_HUB_TOKEN)
        #[arg(long)]
        token: Option<String>,
        /// Output path (default: .tellur/policies/<name>.yml)
        #[arg(long)]
        out: Option<PathBuf>,
    },
}
