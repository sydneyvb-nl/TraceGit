//! Policy commands: `policy check`, `policy explain`, and `policy pull`.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use tellur_core::schema::types::{PolicyResult, RiskLevel};
use tellur_core::storage::{RepoStorage, TraceIndex};

#[derive(Serialize)]
struct PolicyFinding {
    file_path: String,
    start_line: u32,
    end_line: u32,
    rule_id: String,
    severity: RiskLevel,
    message: String,
    evidence: Vec<String>,
}

#[derive(Serialize)]
struct PolicyCheckReport {
    passed: bool,
    attributions_checked: usize,
    findings: Vec<PolicyFinding>,
}

pub(crate) fn cmd_policy_check(json: bool) -> Result<()> {
    let storage = RepoStorage::discover()?;
    if !storage.is_initialized() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&PolicyCheckReport {
                    passed: true,
                    attributions_checked: 0,
                    findings: Vec::new(),
                })?
            );
        } else {
            println!("Tellur not initialized. Run `tellur init` first.");
        }
        return Ok(());
    }

    let policy_path = storage.policies_dir.join("default.yml");
    if !policy_path.exists() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&PolicyCheckReport {
                    passed: true,
                    attributions_checked: 0,
                    findings: Vec::new(),
                })?
            );
        } else {
            println!("No policy file found.");
        }
        return Ok(());
    }

    let engine = tellur_core::policy::PolicyEngine::load_from_file(&policy_path)?;
    let attributions = TraceIndex::open(&storage.index_path)?.list_attributions()?;
    let findings = attributions
        .iter()
        .flat_map(|item| {
            engine
                .evaluate_attribution(&item.range, &item.file_path)
                .into_iter()
                .filter(|result| !result.passed)
                .map(|result| finding_from_result(item, result))
        })
        .collect::<Vec<_>>();
    let report = PolicyCheckReport {
        passed: findings.is_empty(),
        attributions_checked: attributions.len(),
        findings,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_policy_summary(engine.policy());
        println!();
        println!(
            "Checked {} attribution range{}.",
            report.attributions_checked,
            if report.attributions_checked == 1 {
                ""
            } else {
                "s"
            }
        );

        if report.passed {
            println!("✓ No policy violations found.");
        } else {
            println!("Policy violations ({}):", report.findings.len());
            for finding in &report.findings {
                println!(
                    "  {} {} — {}:{}-{}",
                    risk_level_label(&finding.severity),
                    finding.rule_id,
                    finding.file_path,
                    finding.start_line,
                    finding.end_line
                );
                println!("    {}", finding.message);
                for evidence in &finding.evidence {
                    println!("    Evidence: {evidence}");
                }
            }
        }
    }

    if report.passed {
        Ok(())
    } else {
        anyhow::bail!(
            "policy check failed with {} violation{}",
            report.findings.len(),
            if report.findings.len() == 1 { "" } else { "s" }
        )
    }
}

fn finding_from_result(
    item: &tellur_core::notes::IndexedAttribution,
    result: PolicyResult,
) -> PolicyFinding {
    PolicyFinding {
        file_path: item.file_path.clone(),
        start_line: item.range.start_line,
        end_line: item.range.end_line,
        rule_id: result.rule_id,
        severity: result.severity,
        message: result.message,
        evidence: result.evidence,
    }
}

fn print_policy_summary(policy: &tellur_core::schema::types::PolicyFile) {
    println!("Policy Check");
    println!("════════════");
    println!();

    if let Some(paths) = &policy.sensitive_paths {
        println!("Sensitive paths ({}):", paths.len());
        for path in paths {
            println!("  {} [{}]", path.path, path.tags.join(", "));
        }
    }

    if let Some(rules) = &policy.rules {
        if rules.is_empty() {
            println!("Custom rules: none");
        } else {
            println!("Custom rules ({}):", rules.len());
            for rule in rules {
                println!("  {} — {}", rule.id, rule.description);
            }
        }
    }
}

fn risk_level_label(level: &RiskLevel) -> &'static str {
    match level {
        RiskLevel::Low => "LOW",
        RiskLevel::Medium => "MEDIUM",
        RiskLevel::High => "HIGH",
        RiskLevel::Critical => "CRITICAL",
    }
}

pub(crate) fn cmd_policy_explain(rule_id: Option<&str>) -> Result<()> {
    let storage = RepoStorage::discover()?;
    let policy_path = storage.policies_dir.join("default.yml");
    if !policy_path.exists() {
        println!("No policy file found.");
        return Ok(());
    }

    let engine = tellur_core::policy::PolicyEngine::load_from_file(&policy_path)?;
    let policy = engine.policy();

    if let Some(id) = rule_id {
        if let Some(ref rules) = policy.rules {
            if let Some(rule) = rules.iter().find(|r| r.id == id) {
                println!("Rule: {}", rule.id);
                println!("Description: {}", rule.description);
                if let Some(ref rationale) = rule.rationale {
                    println!("Rationale: {}", rationale);
                }
                println!("Action: {:?}", rule.action);
                println!("When: {}", serde_json::to_string_pretty(&rule.when)?);
            } else {
                println!("Rule '{}' not found.", id);
            }
        }
    } else {
        println!("Available rules:");
        if let Some(ref rules) = policy.rules {
            for rule in rules {
                println!("  {} — {}", rule.id, rule.description);
            }
        }
        if policy.rules.is_none() || policy.rules.as_ref().map(|r| r.is_empty()).unwrap_or(true) {
            println!("  (no custom rules defined)");
        }
    }

    Ok(())
}

/// Pull a central policy from a Tellur team hub (Tier 0/Tier 1 distribution) and
/// write it into this repo's `.tellur/policies/`. Validates the content before
/// writing so a broken policy is never installed.
pub(crate) fn cmd_policy_pull(
    org: &str,
    name: &str,
    hub: Option<&str>,
    token: Option<&str>,
    out: Option<&Path>,
) -> Result<()> {
    let hub = hub
        .map(str::to_string)
        .or_else(|| std::env::var("TELLUR_HUB_URL").ok())
        .context("hub URL required (--hub or TELLUR_HUB_URL)")?;
    let token = token
        .map(str::to_string)
        .or_else(|| std::env::var("TELLUR_HUB_TOKEN").ok())
        .context("hub token required (--token or TELLUR_HUB_TOKEN)")?;

    let url = format!(
        "{}/v1/orgs/{}/policies/{}",
        hub.trim_end_matches('/'),
        org,
        name
    );
    let body = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .call()
        .map_err(|e| anyhow::anyhow!("policy pull request failed: {e}"))?
        .into_string()
        .context("failed to read hub response")?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).context("hub response was not valid JSON")?;
    let content = parsed["content"]
        .as_str()
        .context("hub response missing policy content")?;

    // Validate before writing — never install a broken policy.
    tellur_core::policy::PolicyEngine::from_yaml_str(content)
        .context("hub returned invalid policy YAML")?;

    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => {
            let storage = RepoStorage::discover()?;
            storage.policies_dir.join(format!("{name}.yml"))
        }
    };
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, content)?;
    println!(
        "Pulled policy '{}' (version {}) → {}",
        name,
        parsed["version"],
        out_path.display()
    );
    Ok(())
}
