//! Tellur CLI — AI Code Provenance from the terminal.
//!
//! `main` only parses arguments and dispatches; every command lives in a
//! focused module:
//!
//! - [`cli`] — clap argument/subcommand definitions
//! - [`repo`] — `init`, `doctor`
//! - [`inspect`] — `status`, `explain`, `blame`, `pr-report`, `verify`, `sessions`
//! - [`capture`] — `watch`, `event`
//! - [`maintain`] — `export`, `import`, `gc`, `redact`
//! - [`policy`] — `policy check|explain|pull`
//! - [`push`] — `login`, `logout`, `push`
//! - [`notes`] — `notes …`, `team report`
//! - [`connect`] — `connect` zero-touch setup + managed git hooks
//! - [`setup`] — global editor/agent integrations
//! - [`hooks`] — `hooks install|claude|ingest`
//! - [`serve`] — `daemon`, `mcp`
//! - [`util`], [`git`] — cross-cutting helpers shared by the above

use anyhow::Result;
use clap::Parser;

mod capture;
mod cli;
mod connect;
mod git;
mod hooks;
mod hub;
mod inspect;
mod maintain;
mod notes;
mod onboarding;
mod policy;
mod push;
mod repo;
mod serve;
mod service;
mod setup;
mod util;

use cli::{Cli, Commands, HookActions, NotesActions, PolicyActions, SetupActions, TeamActions};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { profile } => repo::cmd_init(&profile).await,
        Commands::Doctor => repo::cmd_doctor().await,
        Commands::Status => inspect::cmd_status(),
        Commands::Explain { target, json } => inspect::cmd_explain(&target, json),
        Commands::Blame { file, json } => inspect::cmd_blame(&file, json),
        Commands::PrReport { base, head } => inspect::cmd_pr_report(&base, &head),
        Commands::Policy { action } => match action {
            PolicyActions::Check { json } => policy::cmd_policy_check(json),
            PolicyActions::Explain { rule_id } => policy::cmd_policy_explain(rule_id.as_deref()),
            PolicyActions::Pull {
                org,
                name,
                hub,
                token,
                out,
            } => policy::cmd_policy_pull(
                &org,
                &name,
                hub.as_deref(),
                token.as_deref(),
                out.as_deref(),
            ),
        },
        Commands::Connect {
            hub,
            remote,
            no_login,
            no_agents,
            background,
            push_interval,
            no_browser,
            status,
            remove,
        } => connect::cmd_connect(connect::ConnectOptions {
            hub: hub.as_deref(),
            remote: &remote,
            no_login,
            no_agents,
            background,
            push_interval,
            no_browser,
            status,
            remove,
        }),
        Commands::Login { hub, no_browser } => push::cmd_login(hub.as_deref(), no_browser),
        Commands::Logout { hub } => push::cmd_logout(hub.as_deref()),
        Commands::Push {
            hub,
            org,
            repo,
            token,
            dry_run,
            reset,
        } => push::cmd_push(
            hub.as_deref(),
            org.as_deref(),
            repo.as_deref(),
            token.as_deref(),
            dry_run,
            reset,
        ),
        Commands::Export { format, output } => maintain::cmd_export(&format, output.as_deref()),
        Commands::Import { adapter, source } => maintain::cmd_import(&adapter, &source).await,
        Commands::Watch {
            agent_id,
            agent_name,
            model_id,
        } => capture::cmd_watch(&agent_id, &agent_name, model_id).await,
        Commands::Event {
            event_type,
            session,
            file,
            command,
            exit_code,
            payload_json,
        } => capture::cmd_event(
            &event_type,
            &session,
            file.as_deref(),
            command.as_deref(),
            exit_code,
            payload_json.as_deref(),
        ),
        Commands::Gc { dry_run } => maintain::cmd_gc(dry_run),
        Commands::Verify => inspect::cmd_verify(),
        Commands::Redact => maintain::cmd_redact(),
        Commands::Sessions { session_id, json } => {
            inspect::cmd_sessions(session_id.as_deref(), json)
        }
        Commands::Daemon { host, port } => serve::cmd_daemon(&host, port).await,
        Commands::Mcp => serve::cmd_mcp(),
        Commands::Notes { action } => match action {
            NotesActions::Export {
                commit,
                notes_ref,
                print,
            } => notes::cmd_notes_export(&commit, &notes_ref, print),
            NotesActions::AttestAi {
                commit,
                session,
                agent,
                model,
                notes_ref,
                force,
            } => notes::cmd_notes_attest_ai(&commit, &notes_ref, &session, &agent, &model, force),
            NotesActions::Show {
                commit,
                notes_ref,
                json,
            } => notes::cmd_notes_show(&commit, &notes_ref, json),
            NotesActions::Import { commit, notes_ref } => {
                notes::cmd_notes_import(&commit, &notes_ref)
            }
            NotesActions::Fetch { remote, notes_ref } => {
                notes::cmd_notes_fetch(&remote, &notes_ref)
            }
            NotesActions::Push { remote, notes_ref } => notes::cmd_notes_push(&remote, &notes_ref),
            NotesActions::InstallConfig { remote, notes_ref } => {
                notes::cmd_notes_install_config(&remote, &notes_ref)
            }
        },
        Commands::Team { action } => match action {
            TeamActions::Report {
                base,
                head,
                notes_ref,
                json,
            } => notes::cmd_team_report(&base, &head, &notes_ref, json),
        },
        Commands::Hooks { action } => match action {
            HookActions::Install { tool } => hooks::cmd_hooks_install(&tool),
            HookActions::Claude => hooks::cmd_hooks_claude(),
            HookActions::Ingest {
                source,
                auto_init,
                json_response,
            } => hooks::cmd_hooks_ingest(&source, auto_init, json_response),
        },
        Commands::Setup {
            action,
            hub,
            local_only,
            remote,
            no_background,
            no_browser,
            yes,
        } => match action {
            None => onboarding::cmd_setup(onboarding::SetupOptions {
                hub: hub.as_deref(),
                local_only,
                remote: &remote,
                no_background,
                no_browser,
                yes,
                update: false,
            }),
            Some(SetupActions::Update) => onboarding::cmd_setup(onboarding::SetupOptions {
                hub: hub.as_deref(),
                local_only,
                remote: &remote,
                no_background,
                no_browser,
                yes: true,
                update: true,
            }),
            Some(SetupActions::Agents { home }) => setup::cmd_setup_agents(home.as_deref()),
            Some(SetupActions::Codex { home }) => setup::cmd_setup_codex(home.as_deref()),
            Some(SetupActions::ClaudeCode { home }) => {
                setup::cmd_setup_claude_code(home.as_deref())
            }
            Some(SetupActions::Cursor { home }) => setup::cmd_setup_cursor(home.as_deref()),
            Some(SetupActions::Vscode { home }) => setup::cmd_setup_vscode(home.as_deref()),
            Some(SetupActions::Windsurf { home }) => setup::cmd_setup_windsurf(home.as_deref()),
            Some(SetupActions::GeminiCli { home }) => setup::cmd_setup_gemini_cli(home.as_deref()),
            Some(SetupActions::Antigravity { home }) => {
                setup::cmd_setup_antigravity(home.as_deref())
            }
            Some(SetupActions::Status { home, remote }) => {
                onboarding::cmd_setup_status(home.as_deref(), &remote)
            }
            Some(SetupActions::Uninstall { home }) => setup::cmd_setup_uninstall(home.as_deref()),
        },
    }
}
