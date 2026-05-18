#![forbid(unsafe_code)]
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REQUIRED_DOCS: &[&str] = &[
    "README.md",
    "docs/MISSION.md",
    "docs/ROADMAP.md",
    "docs/specs/README.md",
    "docs/status/SUPPORT_TIERS.md",
];

const POLICY_FILES: &[&str] = &[
    "policy/unsafe-review.toml",
    "policy/unsafe-review-baseline.toml",
    "policy/unsafe-review-suppressions.toml",
    "policy/clippy-lints.toml",
    "policy/clippy-exceptions.toml",
    "policy/no-panic-allowlist.toml",
    "policy/non-rust-allowlist.toml",
    "policy/generated-allowlist.toml",
    "policy/executable-allowlist.toml",
    "policy/workflow-allowlist.toml",
    "policy/process-allowlist.toml",
    "policy/network-allowlist.toml",
];

const FIXTURE_REQUIRED_FILES: &[&str] = &["Cargo.toml", "change.diff", "src/lib.rs"];

const FIXTURE_EXPECTED_CARDS_EXCEPTIONS: &[&str] = &[
    "duplicate_raw_pointer_reads",
    "raw_pointer_alignment_line_drift",
];

const FIXTURE_PACKAGE_PREFIX_EXCEPTIONS: &[(&str, &str)] =
    &[("raw_pointer_alignment_line_drift", "raw-pointer-alignment")];

const CALIBRATION_REQUIRED_KINDS: &[&str] = &["positive", "negative", "false_positive_control"];

const KNOWN_SUPPORT_TIERS: &[&str] = &["scaffold", "experimental", "planned", "deferred"];
const DOGFOOD_MANIFEST: &str = "docs/dogfood/corpus.toml";
const DOGFOOD_TARGET_KINDS: &[&str] = &["repo-snapshot", "pr-diff"];
const DOGFOOD_TARGET_STATUSES: &[&str] = &["active", "parked", "retired"];
const DOGFOOD_ARTIFACT_STATUSES: &[&str] = &["checked_in", "local_untracked", "remote_manual"];

fn main() {
    if let Err(err) = run(std::env::args().collect()) {
        eprintln!("xtask: {err}");
        std::process::exit(2);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.get(1).map(|arg| arg.as_str()) {
        None | Some("help") | Some("--help") => {
            println!(
                "xtask commands: check-pr, check-docs, check-policy, check-support-tiers, check-fixtures, check-calibration, check-dogfood, check-advisory-artifacts <dir>"
            );
            Ok(())
        }
        Some("check-pr") => {
            check_docs()?;
            check_policy()?;
            check_support_tiers()?;
            check_fixtures()?;
            check_calibration()?;
            check_dogfood()?;
            check_tracked_generated_artifacts()?;
            println!("check-pr: ok");
            Ok(())
        }
        Some("check-docs") => check_docs(),
        Some("check-policy") => check_policy(),
        Some("check-support-tiers") => check_support_tiers(),
        Some("check-fixtures") => check_fixtures(),
        Some("check-calibration") => check_calibration(),
        Some("check-dogfood") => check_dogfood(),
        Some("check-advisory-artifacts") => {
            let Some(dir) = args.get(2) else {
                return Err("usage: cargo xtask check-advisory-artifacts <dir>".to_string());
            };
            check_advisory_artifacts(Path::new(dir))
        }
        Some(other) => Err(format!("unknown xtask command `{other}`")),
    }
}

fn check_docs() -> Result<(), String> {
    for path in REQUIRED_DOCS {
        require_file(path)?;
    }
    check_index(
        Path::new("docs/specs"),
        Path::new("docs/specs/README.md"),
        "UNSAFE-REVIEW-SPEC-",
    )?;
    check_index(
        Path::new("docs/adr"),
        Path::new("docs/adr/README.md"),
        "UNSAFE-REVIEW-ADR-",
    )?;
    check_index(
        Path::new("docs/proposals"),
        Path::new("docs/proposals/README.md"),
        "UNSAFE-REVIEW-PROP-",
    )?;
    check_no_windows_paths(&[
        Path::new("README.md"),
        Path::new("MANIFEST.md"),
        Path::new("docs"),
        Path::new("plans"),
        Path::new(".unsafe-review"),
        Path::new("policy"),
    ])?;
    println!("check-docs: ok");
    Ok(())
}

fn check_policy() -> Result<(), String> {
    for path in POLICY_FILES {
        let value = parse_toml_file(Path::new(path))?;
        require_toml_string(&value, "schema_version", path)?;
    }
    check_unsafe_review_ledger(
        Path::new("policy/unsafe-review-baseline.toml"),
        LedgerKind::Baseline,
    )?;
    check_unsafe_review_ledger(
        Path::new("policy/unsafe-review-suppressions.toml"),
        LedgerKind::Suppression,
    )?;
    parse_toml_file(Path::new(".unsafe-review/goals/active.toml"))?;
    println!("check-policy: ok");
    Ok(())
}

#[derive(Clone, Copy)]
enum LedgerKind {
    Baseline,
    Suppression,
}

impl LedgerKind {
    fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::Suppression => "suppression",
        }
    }
}

fn check_unsafe_review_ledger(path: &Path, kind: LedgerKind) -> Result<(), String> {
    let value = parse_toml_file(path)?;
    let path_display = path.display().to_string();
    let status = value
        .get("status")
        .and_then(toml::Value::as_str)
        .unwrap_or("active");
    let entries = value
        .get("entries")
        .and_then(toml::Value::as_array)
        .map_or(&[][..], Vec::as_slice);

    if status == "empty" {
        if entries.is_empty() {
            return Ok(());
        }
        return Err(format!(
            "{path_display} status is empty but contains entries"
        ));
    }

    for (idx, entry) in entries.iter().enumerate() {
        let Some(entry) = entry.as_table() else {
            return Err(format!(
                "{path_display} entries[{idx}] must be a TOML table"
            ));
        };
        for key in ["card_id", "owner", "reason", "evidence"] {
            require_ledger_entry_string(entry, key, &path_display, idx)?;
        }
        let has_review_after = ledger_entry_date(entry, "review_after", &path_display, idx)?;
        let has_expires = ledger_entry_date(entry, "expires", &path_display, idx)?;
        match kind {
            LedgerKind::Baseline if !has_review_after => {
                return Err(format!(
                    "{path_display} entries[{idx}] baseline entry is missing review_after"
                ));
            }
            LedgerKind::Suppression if !has_review_after && !has_expires => {
                return Err(format!(
                    "{path_display} entries[{idx}] suppression entry must set review_after or expires"
                ));
            }
            _ => {}
        }
        let card_id = entry
            .get("card_id")
            .and_then(toml::Value::as_str)
            .unwrap_or_default();
        if !looks_like_counted_card_id(card_id) {
            return Err(format!(
                "{path_display} entries[{idx}] {} card_id must be an exact counted UR-* identity ending in -cN",
                kind.name()
            ));
        }
    }

    Ok(())
}

fn require_ledger_entry_string(
    entry: &toml::map::Map<String, toml::Value>,
    key: &str,
    path: &str,
    idx: usize,
) -> Result<(), String> {
    let Some(value) = entry.get(key).and_then(toml::Value::as_str) else {
        return Err(format!("{path} entries[{idx}] is missing string `{key}`"));
    };
    if value.trim().is_empty() {
        Err(format!("{path} entries[{idx}] string `{key}` is empty"))
    } else {
        Ok(())
    }
}

fn ledger_entry_date(
    entry: &toml::map::Map<String, toml::Value>,
    key: &str,
    path: &str,
    idx: usize,
) -> Result<bool, String> {
    let Some(value) = entry.get(key) else {
        return Ok(false);
    };
    let Some(value) = value.as_str() else {
        return Err(format!("{path} entries[{idx}] `{key}` must be a string"));
    };
    if !looks_like_iso_date(value) {
        return Err(format!("{path} entries[{idx}] `{key}` must use YYYY-MM-DD"));
    }
    Ok(true)
}

fn looks_like_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn looks_like_counted_card_id(value: &str) -> bool {
    let Some((prefix, count)) = value.rsplit_once("-c") else {
        return false;
    };
    value.starts_with("UR-")
        && !prefix.is_empty()
        && !count.is_empty()
        && count.bytes().all(|byte| byte.is_ascii_digit())
}

fn check_support_tiers() -> Result<(), String> {
    let path = "docs/status/SUPPORT_TIERS.md";
    let text = read_to_string(Path::new(path))?;
    let mut rows = 0usize;
    for (line_no, line) in text.lines().enumerate() {
        let Some(tier) = support_tier_from_row(line) else {
            continue;
        };
        rows += 1;
        if !KNOWN_SUPPORT_TIERS.contains(&tier) {
            return Err(format!(
                "{path}:{} uses unknown support tier `{tier}`",
                line_no + 1
            ));
        }
    }
    if rows == 0 {
        return Err(format!("{path} has no support-tier rows"));
    }
    println!("check-support-tiers: ok");
    Ok(())
}

fn check_fixtures() -> Result<(), String> {
    let dirs = fixture_dirs(Path::new("fixtures"))?;
    if dirs.is_empty() {
        return Err("fixtures directory has no fixture cases".to_string());
    }
    for dir in &dirs {
        check_fixture(dir)?;
    }
    println!("check-fixtures: ok ({} fixtures)", dirs.len());
    Ok(())
}

fn check_calibration() -> Result<(), String> {
    let path = workspace_path("fixtures/calibration.toml");
    let value = parse_toml_file(&path)?;
    require_toml_string(&value, "schema_version", "fixtures/calibration.toml")?;
    let required = value
        .get("required_core_fixtures")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "fixtures/calibration.toml is missing required_core_fixtures".to_string())?;
    let cases = value
        .get("cases")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "fixtures/calibration.toml is missing cases".to_string())?;
    if cases.is_empty() {
        return Err("fixtures/calibration.toml has no calibration cases".to_string());
    }

    let mut fixtures = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for (idx, case) in cases.iter().enumerate() {
        let Some(case) = case.as_table() else {
            return Err(format!(
                "fixtures/calibration.toml cases[{idx}] must be a TOML table"
            ));
        };
        let fixture = required_case_string(case, "fixture", idx)?;
        let kind = required_case_string(case, "kind", idx)?;
        let claim = required_case_string(case, "claim", idx)?;
        required_case_string(case, "support_tier", idx)?;
        if !CALIBRATION_REQUIRED_KINDS.contains(&kind) {
            return Err(format!(
                "fixtures/calibration.toml cases[{idx}] uses unknown kind `{kind}`"
            ));
        }
        if claim.len() < 16 {
            return Err(format!(
                "fixtures/calibration.toml cases[{idx}] claim is too terse"
            ));
        }
        if !fixtures.insert(fixture.to_string()) {
            return Err(format!(
                "fixtures/calibration.toml contains duplicate fixture `{fixture}`"
            ));
        }
        kinds.insert(kind.to_string());
        check_calibration_case(case, fixture, idx)?;
    }

    for kind in CALIBRATION_REQUIRED_KINDS {
        if !kinds.contains(*kind) {
            return Err(format!(
                "fixtures/calibration.toml is missing a `{kind}` calibration case"
            ));
        }
    }

    for (idx, fixture) in required.iter().enumerate() {
        let Some(fixture) = fixture.as_str() else {
            return Err(format!(
                "fixtures/calibration.toml required_core_fixtures[{idx}] must be a string"
            ));
        };
        if !fixtures.contains(fixture) {
            return Err(format!(
                "fixtures/calibration.toml required core fixture `{fixture}` has no case"
            ));
        }
    }

    println!("check-calibration: ok ({} cases)", cases.len());
    Ok(())
}

fn check_dogfood() -> Result<(), String> {
    let value = parse_toml_file(&workspace_path(DOGFOOD_MANIFEST))?;
    require_toml_string(&value, "schema_version", DOGFOOD_MANIFEST)?;
    require_toml_string(&value, "status", DOGFOOD_MANIFEST)?;
    require_toml_string(&value, "artifact_root", DOGFOOD_MANIFEST)?;
    let boundary = required_toml_string(&value, "trust_boundary", DOGFOOD_MANIFEST)?;
    require_boundary_text(boundary, DOGFOOD_MANIFEST)?;

    let targets = value
        .get("targets")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| format!("{DOGFOOD_MANIFEST} is missing targets"))?;
    if targets.is_empty() {
        return Err(format!("{DOGFOOD_MANIFEST} has no dogfood targets"));
    }

    let mut ids = BTreeSet::new();
    let mut repositories = BTreeSet::new();
    let mut repo_snapshots = 0usize;
    let mut pr_diffs = 0usize;
    for (idx, target) in targets.iter().enumerate() {
        let Some(target) = target.as_table() else {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] must be a TOML table"
            ));
        };
        let id = required_target_string(target, "id", idx)?;
        if !ids.insert(id.to_string()) {
            return Err(format!(
                "{DOGFOOD_MANIFEST} contains duplicate target id `{id}`"
            ));
        }
        let repository = required_target_string(target, "repository", idx)?;
        if !repository.contains('/') {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] repository `{repository}` must be owner/repo"
            ));
        }
        repositories.insert(repository.to_string());
        required_target_string(target, "crate", idx)?;
        let kind = required_target_string(target, "kind", idx)?;
        if !DOGFOOD_TARGET_KINDS.contains(&kind) {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] uses unknown kind `{kind}`"
            ));
        }
        let status = required_target_string(target, "status", idx)?;
        if !DOGFOOD_TARGET_STATUSES.contains(&status) {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] uses unknown status `{status}`"
            ));
        }
        let purpose = required_target_string(target, "purpose", idx)?;
        if purpose.len() < 24 {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] purpose is too terse"
            ));
        }
        let command = required_target_string(target, "command", idx)?;
        if !command.contains("unsafe-review") || !command.contains("--format json") {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] command must run unsafe-review JSON output"
            ));
        }
        let artifact_status = required_target_string(target, "artifact_status", idx)?;
        if !DOGFOOD_ARTIFACT_STATUSES.contains(&artifact_status) {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] uses unknown artifact_status `{artifact_status}`"
            ));
        }
        let artifacts = target
            .get("artifacts")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| {
                format!("{DOGFOOD_MANIFEST} targets[{idx}] is missing artifacts array")
            })?;
        if artifacts.is_empty() {
            return Err(format!(
                "{DOGFOOD_MANIFEST} targets[{idx}] artifacts array is empty"
            ));
        }
        for (artifact_idx, artifact) in artifacts.iter().enumerate() {
            let Some(artifact) = artifact.as_str() else {
                return Err(format!(
                    "{DOGFOOD_MANIFEST} targets[{idx}] artifacts[{artifact_idx}] must be a string"
                ));
            };
            check_dogfood_path(artifact, idx, "artifacts")?;
            if artifact_status == "checked_in" && !Path::new(artifact).is_file() {
                return Err(format!(
                    "{DOGFOOD_MANIFEST} targets[{idx}] checked-in artifact missing: {artifact}"
                ));
            }
        }

        match kind {
            "repo-snapshot" => {
                repo_snapshots += 1;
                let commit = required_target_string(target, "commit", idx)?;
                if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err(format!(
                        "{DOGFOOD_MANIFEST} targets[{idx}] commit must be a full 40-character hex SHA"
                    ));
                }
                let root = required_target_string(target, "root", idx)?;
                check_dogfood_path(root, idx, "root")?;
            }
            "pr-diff" => {
                pr_diffs += 1;
                let Some(pr) = target.get("pr").and_then(toml::Value::as_integer) else {
                    return Err(format!(
                        "{DOGFOOD_MANIFEST} targets[{idx}] is missing integer pr"
                    ));
                };
                if pr <= 0 {
                    return Err(format!(
                        "{DOGFOOD_MANIFEST} targets[{idx}] pr must be positive"
                    ));
                }
                let root = required_target_string(target, "root", idx)?;
                check_dogfood_path(root, idx, "root")?;
                let diff = required_target_string(target, "diff", idx)?;
                check_dogfood_path(diff, idx, "diff")?;
            }
            _ => {
                return Err(format!(
                    "{DOGFOOD_MANIFEST} targets[{idx}] uses unsupported kind `{kind}`"
                ));
            }
        }
    }

    if repositories.len() < 5 {
        return Err(format!(
            "{DOGFOOD_MANIFEST} must cover at least 5 real repositories"
        ));
    }
    if repo_snapshots == 0 || pr_diffs == 0 {
        return Err(format!(
            "{DOGFOOD_MANIFEST} must include repo-snapshot and pr-diff targets"
        ));
    }

    println!(
        "check-dogfood: ok ({} targets, {} repositories)",
        targets.len(),
        repositories.len()
    );
    Ok(())
}

fn check_advisory_artifacts(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Err(format!(
            "advisory artifact directory missing: {}",
            dir.display()
        ));
    }

    let cards = parse_json_file(&dir.join("cards.json"))?;
    require_json_str(&cards, "tool", "unsafe-review", "cards.json")?;
    require_json_str(&cards, "policy", "advisory", "cards.json")?;
    require_json_array(&cards, "cards", "cards.json")?;
    let cards_boundary = cards
        .get("trust_boundary")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cards.json is missing trust_boundary".to_string())?;
    require_boundary_text(cards_boundary, "cards.json")?;
    let card_ids = advisory_card_ids(&cards)?;
    let card_count = card_ids.len();
    let summary_cards = json_usize_at(&cards, "/summary/cards", "cards.json")?;
    if summary_cards != card_count {
        return Err(format!(
            "cards.json summary.cards is {summary_cards}, but cards array has {card_count}"
        ));
    }

    let pr_summary_path = dir.join("pr-summary.md");
    let pr_summary = read_to_string(&pr_summary_path)?;
    require_text_contains(
        &pr_summary,
        &format!("- Review cards: {card_count}"),
        &pr_summary_path,
    )?;
    require_text_contains(
        &pr_summary,
        "static unsafe contract review",
        &pr_summary_path,
    )?;
    require_text_contains(
        &pr_summary,
        "not a proof of memory safety",
        &pr_summary_path,
    )?;
    require_text_contains(&pr_summary, "not UB-free status", &pr_summary_path)?;
    require_text_contains(&pr_summary, "not a Miri result", &pr_summary_path)?;

    let sarif = parse_json_file(&dir.join("cards.sarif"))?;
    require_json_str(&sarif, "version", "2.1.0", "cards.sarif")?;
    require_json_array(&sarif, "runs", "cards.sarif")?;
    let sarif_results = json_array_at(&sarif, "/runs/0/results", "cards.sarif")?;
    if sarif_results.len() != card_count {
        return Err(format!(
            "cards.sarif has {} result(s), but cards.json has {card_count} card(s)",
            sarif_results.len()
        ));
    }
    for result in sarif_results {
        let Some(card_id) = result
            .pointer("/properties/cardId")
            .and_then(serde_json::Value::as_str)
        else {
            return Err("cards.sarif result is missing properties.cardId".to_string());
        };
        if !card_ids.contains(card_id) {
            return Err(format!(
                "cards.sarif result references unknown card id `{card_id}`"
            ));
        }
    }
    let sarif_boundary = sarif
        .pointer("/runs/0/properties/trustBoundary")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "cards.sarif is missing /runs/0/properties/trustBoundary".to_string())?;
    require_boundary_text(sarif_boundary, "cards.sarif")?;

    let comment_plan = parse_json_file(&dir.join("comment-plan.json"))?;
    require_json_str(&comment_plan, "mode", "plan_only", "comment-plan.json")?;
    require_json_str(&comment_plan, "policy", "advisory", "comment-plan.json")?;
    require_json_array(&comment_plan, "comments", "comment-plan.json")?;
    for comment in json_array_at(&comment_plan, "/comments", "comment-plan.json")? {
        let Some(card_id) = comment.get("card_id").and_then(serde_json::Value::as_str) else {
            return Err("comment-plan.json comment is missing card_id".to_string());
        };
        if !card_ids.contains(card_id) {
            return Err(format!(
                "comment-plan.json references unknown card id `{card_id}`"
            ));
        }
    }
    let comment_boundary = comment_plan
        .get("trust_boundary")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "comment-plan.json is missing trust_boundary".to_string())?;
    require_boundary_text(comment_boundary, "comment-plan.json")?;

    println!("check-advisory-artifacts: ok ({})", dir.display());
    Ok(())
}

fn check_fixture(dir: &Path) -> Result<(), String> {
    let name = fixture_dir_name(dir)?;
    if !is_snake_case_name(name) {
        return Err(format!(
            "{} must use a lowercase snake_case fixture name",
            dir.display()
        ));
    }

    for relative in FIXTURE_REQUIRED_FILES {
        require_fixture_file(dir, relative)?;
    }

    let cargo = parse_toml_file(&dir.join("Cargo.toml"))?;
    let package_name = cargo
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("{}/Cargo.toml is missing package.name", dir.display()))?;
    let expected_prefix = fixture_package_prefix(name);
    if !package_name.starts_with(&expected_prefix) {
        return Err(format!(
            "{}/Cargo.toml package.name `{package_name}` does not start with `{expected_prefix}`",
            dir.display()
        ));
    }

    let expected_cards = dir.join("expected.cards.json");
    if expected_cards.is_file() {
        let expected_cards = parse_json_file(&expected_cards)?;
        if !expected_cards.is_array() {
            return Err(format!(
                "{}/expected.cards.json must contain a JSON array of cards",
                dir.display()
            ));
        }
    } else if !FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&name) {
        return Err(format!(
            "fixture {} is missing expected.cards.json",
            dir.display()
        ));
    }

    let diff = read_to_string(&dir.join("change.diff"))?;
    if !looks_like_git_diff(&diff) {
        return Err(format!(
            "{}/change.diff does not look like a unified git diff",
            dir.display()
        ));
    }

    Ok(())
}

fn check_calibration_case(
    case: &toml::map::Map<String, toml::Value>,
    fixture: &str,
    idx: usize,
) -> Result<(), String> {
    let fixture_dir = workspace_path("fixtures").join(fixture);
    if !fixture_dir.is_dir() {
        return Err(format!(
            "fixtures/calibration.toml cases[{idx}] references missing fixture `{fixture}`"
        ));
    }
    let expected_cards = required_case_usize(case, "expected_cards", idx)?;
    let cards = parse_json_file(&fixture_dir.join("expected.cards.json"))?;
    let Some(cards) = cards.as_array() else {
        return Err(format!(
            "{}/expected.cards.json must contain a JSON array",
            fixture_dir.display()
        ));
    };
    if cards.len() != expected_cards {
        return Err(format!(
            "fixtures/calibration.toml cases[{idx}] expected_cards is {expected_cards}, but {fixture}/expected.cards.json has {} card(s)",
            cards.len()
        ));
    }
    if expected_cards == 0 {
        return Ok(());
    }
    let expected_class = required_case_string(case, "expected_class", idx)?;
    if !cards
        .iter()
        .any(|card| json_str(card, "class") == Some(expected_class))
    {
        return Err(format!(
            "fixtures/calibration.toml cases[{idx}] expected_class `{expected_class}` was not found in {fixture}/expected.cards.json"
        ));
    }
    if let Some(expected_operation_family) = case
        .get("expected_operation_family")
        .and_then(toml::Value::as_str)
        && !cards
            .iter()
            .any(|card| json_str(card, "operation_family") == Some(expected_operation_family))
    {
        return Err(format!(
            "fixtures/calibration.toml cases[{idx}] expected_operation_family `{expected_operation_family}` was not found in {fixture}/expected.cards.json"
        ));
    }
    if let Some(expected_hazard) = case.get("expected_hazard").and_then(toml::Value::as_str)
        && !cards
            .iter()
            .any(|card| json_array_contains_str(card, "hazards", expected_hazard))
    {
        return Err(format!(
            "fixtures/calibration.toml cases[{idx}] expected_hazard `{expected_hazard}` was not found in {fixture}/expected.cards.json"
        ));
    }
    Ok(())
}

fn check_index(dir: &Path, readme: &Path, prefix: &str) -> Result<(), String> {
    let index = read_to_string(readme)?;
    let mut ids = BTreeSet::new();
    for path in markdown_files(dir)? {
        if path == readme {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return Err(format!("non-UTF-8 file name in {}", dir.display()));
        };
        if !name.starts_with(prefix) {
            return Err(format!(
                "{} does not use expected `{prefix}` prefix",
                path.display()
            ));
        }
        let id = name.trim_end_matches(".md");
        if !ids.insert(id.to_string()) {
            return Err(format!("duplicate source-of-truth id `{id}`"));
        }
        if !index.contains(name) {
            return Err(format!(
                "{} is missing from {}",
                path.display(),
                readme.display()
            ));
        }
    }
    Ok(())
}

fn check_no_windows_paths(paths: &[&Path]) -> Result<(), String> {
    for path in paths {
        visit_text(path, &mut |file| {
            let text = read_to_string(file)?;
            for (line_no, line) in text.lines().enumerate() {
                if has_windows_path(line) {
                    return Err(format!(
                        "{}:{} contains a Windows-style path",
                        file.display(),
                        line_no + 1
                    ));
                }
            }
            Ok(())
        })?;
    }
    Ok(())
}

fn check_tracked_generated_artifacts() -> Result<(), String> {
    let output = Command::new("git")
        .args(["ls-files"])
        .output()
        .map_err(|err| format!("failed to run git ls-files: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    for path in String::from_utf8_lossy(&output.stdout).lines() {
        if is_forbidden_generated_path(path) {
            return Err(format!("generated artifact is tracked: {path}"));
        }
    }
    Ok(())
}

fn parse_toml_file(path: &Path) -> Result<toml::Value, String> {
    let text = read_to_string(path)?;
    text.parse::<toml::Value>()
        .map_err(|err| format!("{} is not valid TOML: {err}", path.display()))
}

fn parse_json_file(path: &Path) -> Result<serde_json::Value, String> {
    let text = read_to_string(path)?;
    serde_json::from_str(&text)
        .map_err(|err| format!("{} is not valid JSON: {err}", path.display()))
}

fn advisory_card_ids(cards: &serde_json::Value) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    for card in json_array_at(cards, "/cards", "cards.json")? {
        let Some(id) = card.get("id").and_then(serde_json::Value::as_str) else {
            return Err("cards.json card is missing id".to_string());
        };
        if !ids.insert(id.to_string()) {
            return Err(format!("cards.json contains duplicate card id `{id}`"));
        }
    }
    Ok(ids)
}

fn json_array_at<'a>(
    value: &'a serde_json::Value,
    pointer: &str,
    path: &str,
) -> Result<&'a Vec<serde_json::Value>, String> {
    value
        .pointer(pointer)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("{path} is missing array at `{pointer}`"))
}

fn json_usize_at(value: &serde_json::Value, pointer: &str, path: &str) -> Result<usize, String> {
    let Some(number) = value.pointer(pointer).and_then(serde_json::Value::as_u64) else {
        return Err(format!("{path} is missing unsigned integer at `{pointer}`"));
    };
    usize::try_from(number)
        .map_err(|err| format!("{path} integer at `{pointer}` is too large: {err}"))
}

fn require_toml_string(value: &toml::Value, key: &str, path: &str) -> Result<(), String> {
    match value.get(key).and_then(toml::Value::as_str) {
        Some(_) => Ok(()),
        None => Err(format!("{path} is missing string key `{key}`")),
    }
}

fn required_toml_string<'a>(
    value: &'a toml::Value,
    key: &str,
    path: &str,
) -> Result<&'a str, String> {
    let Some(value) = value.get(key).and_then(toml::Value::as_str) else {
        return Err(format!("{path} is missing string key `{key}`"));
    };
    if value.trim().is_empty() {
        Err(format!("{path} string key `{key}` is empty"))
    } else {
        Ok(value)
    }
}

fn required_target_string<'a>(
    target: &'a toml::map::Map<String, toml::Value>,
    key: &str,
    idx: usize,
) -> Result<&'a str, String> {
    let Some(value) = target.get(key).and_then(toml::Value::as_str) else {
        return Err(format!(
            "{DOGFOOD_MANIFEST} targets[{idx}] is missing string `{key}`"
        ));
    };
    if value.trim().is_empty() {
        Err(format!(
            "{DOGFOOD_MANIFEST} targets[{idx}] string `{key}` is empty"
        ))
    } else {
        Ok(value)
    }
}

fn check_dogfood_path(path: &str, idx: usize, key: &str) -> Result<(), String> {
    if path.starts_with('/') || has_windows_path(path) {
        return Err(format!(
            "{DOGFOOD_MANIFEST} targets[{idx}] {key} path must be relative and use forward slashes: {path}"
        ));
    }
    if path.contains("..") {
        return Err(format!(
            "{DOGFOOD_MANIFEST} targets[{idx}] {key} path must not contain `..`: {path}"
        ));
    }
    Ok(())
}

fn required_case_string<'a>(
    case: &'a toml::map::Map<String, toml::Value>,
    key: &str,
    idx: usize,
) -> Result<&'a str, String> {
    let Some(value) = case.get(key).and_then(toml::Value::as_str) else {
        return Err(format!(
            "fixtures/calibration.toml cases[{idx}] is missing string `{key}`"
        ));
    };
    if value.trim().is_empty() {
        Err(format!(
            "fixtures/calibration.toml cases[{idx}] string `{key}` is empty"
        ))
    } else {
        Ok(value)
    }
}

fn required_case_usize(
    case: &toml::map::Map<String, toml::Value>,
    key: &str,
    idx: usize,
) -> Result<usize, String> {
    let Some(value) = case.get(key).and_then(toml::Value::as_integer) else {
        return Err(format!(
            "fixtures/calibration.toml cases[{idx}] is missing integer `{key}`"
        ));
    };
    usize::try_from(value).map_err(|err| {
        format!("fixtures/calibration.toml cases[{idx}] integer `{key}` is invalid: {err}")
    })
}

fn json_str<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(serde_json::Value::as_str)
}

fn json_array_contains_str(value: &serde_json::Value, key: &str, needle: &str) -> bool {
    value
        .get(key)
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|item| item == needle)
        })
}

fn require_json_str(
    value: &serde_json::Value,
    key: &str,
    expected: &str,
    path: &str,
) -> Result<(), String> {
    match value.get(key).and_then(serde_json::Value::as_str) {
        Some(actual) if actual == expected => Ok(()),
        Some(actual) => Err(format!(
            "{path} key `{key}` is `{actual}`, expected `{expected}`"
        )),
        None => Err(format!("{path} is missing string key `{key}`")),
    }
}

fn require_json_array(value: &serde_json::Value, key: &str, path: &str) -> Result<(), String> {
    if value.get(key).is_some_and(serde_json::Value::is_array) {
        Ok(())
    } else {
        Err(format!("{path} is missing array key `{key}`"))
    }
}

fn require_text_contains(text: &str, needle: &str, path: &Path) -> Result<(), String> {
    if text_contains_ignore_ascii_case(text, needle) {
        Ok(())
    } else {
        Err(format!("{} is missing `{needle}`", path.display()))
    }
}

fn require_boundary_text(text: &str, path: &str) -> Result<(), String> {
    for needle in [
        "static unsafe contract review",
        "not a proof of memory safety",
        "not UB-free status",
        "not a Miri result",
    ] {
        if !text_contains_ignore_ascii_case(text, needle) {
            return Err(format!("{path} trust boundary is missing `{needle}`"));
        }
    }
    Ok(())
}

fn text_contains_ignore_ascii_case(text: &str, needle: &str) -> bool {
    text.to_ascii_lowercase()
        .contains(&needle.to_ascii_lowercase())
}

fn require_file(path: &str) -> Result<(), String> {
    if Path::new(path).is_file() {
        Ok(())
    } else {
        Err(format!("required file missing: {path}"))
    }
}

fn require_fixture_file(dir: &Path, relative: &str) -> Result<(), String> {
    let path = dir.join(relative);
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("fixture {} is missing {relative}", dir.display()))
    }
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
}

fn workspace_path(relative: &str) -> PathBuf {
    let current_dir_path = PathBuf::from(relative);
    if current_dir_path.exists() {
        current_dir_path
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(relative)
    }
}

fn fixture_dirs(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|err| format!("read {} failed: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    visit_text(dir, &mut |path| {
        if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path.to_path_buf());
        }
        Ok(())
    })?;
    files.sort();
    Ok(files)
}

fn visit_text(
    dir_or_file: &Path,
    f: &mut impl FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    if dir_or_file.is_file() {
        if is_text_file(dir_or_file) {
            f(dir_or_file)?;
        }
        return Ok(());
    }
    let entries = fs::read_dir(dir_or_file)
        .map_err(|err| format!("read {} failed: {err}", dir_or_file.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            visit_text(&path, f)?;
        } else if is_text_file(&path) {
            f(&path)?;
        }
    }
    Ok(())
}

fn is_text_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| matches!(ext.to_str(), Some("md" | "toml" | "yml" | "yaml" | "txt")))
}

fn support_tier_from_row(line: &str) -> Option<&str> {
    if !line.starts_with('|') || line.contains("---") || line.contains("Capability") {
        return None;
    }
    let columns = line
        .split('|')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .collect::<Vec<_>>();
    columns.get(1).copied()
}

fn fixture_dir_name(path: &Path) -> Result<&str, String> {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| format!("{} has a non-UTF-8 fixture directory name", path.display()))
}

fn fixture_package_prefix(name: &str) -> String {
    FIXTURE_PACKAGE_PREFIX_EXCEPTIONS
        .iter()
        .find_map(|(fixture, package_prefix)| (*fixture == name).then_some(*package_prefix))
        .unwrap_or(name)
        .replace('_', "-")
}

fn is_snake_case_name(name: &str) -> bool {
    let mut previous_underscore = false;
    let mut has_segment_char = false;
    for ch in name.chars() {
        match ch {
            'a'..='z' | '0'..='9' => {
                previous_underscore = false;
                has_segment_char = true;
            }
            '_' if has_segment_char && !previous_underscore => {
                previous_underscore = true;
            }
            _ => return false,
        }
    }
    has_segment_char && !previous_underscore
}

fn looks_like_git_diff(text: &str) -> bool {
    text.contains("diff --git") && text.contains("--- a/") && text.contains("+++ b/")
}

fn has_windows_path(line: &str) -> bool {
    line.contains(":\\") || line.contains("\\\\")
}

fn is_forbidden_generated_path(path: &str) -> bool {
    path.starts_with("target/")
        || path.starts_with("badges/")
        || path.ends_with(".sarif")
        || path.ends_with(".profraw")
        || path.ends_with(".profdata")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn support_tier_parser_reads_tier_column() {
        assert_eq!(
            support_tier_from_row("| Review cards | scaffold | CLI | proof | limit |"),
            Some("scaffold")
        );
        assert_eq!(support_tier_from_row("|---|---|"), None);
    }

    #[test]
    fn fixture_names_must_be_snake_case() {
        assert!(is_snake_case_name("raw_pointer_deref"));
        assert!(is_snake_case_name("ffi_sanitizer_route"));
        assert!(!is_snake_case_name("RawPointerDeref"));
        assert!(!is_snake_case_name("raw-pointer-deref"));
        assert!(!is_snake_case_name("raw_pointer_deref_"));
        assert!(!is_snake_case_name("_raw_pointer_deref"));
    }

    #[test]
    fn fixture_dir_name_reads_last_path_component() -> Result<(), String> {
        assert_eq!(
            fixture_dir_name(Path::new("fixtures/raw_pointer_deref"))?,
            "raw_pointer_deref"
        );
        Ok(())
    }

    #[test]
    fn expected_card_exceptions_are_exact_fixture_names() {
        assert!(FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&"duplicate_raw_pointer_reads"));
        assert!(FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&"raw_pointer_alignment_line_drift"));
        assert!(!FIXTURE_EXPECTED_CARDS_EXCEPTIONS.contains(&"raw_pointer_alignment"));
    }

    #[test]
    fn calibration_manifest_validates_current_fixture_contract() -> Result<(), String> {
        check_calibration()
    }

    #[test]
    fn dogfood_manifest_validates_current_corpus_contract() -> Result<(), String> {
        check_dogfood()
    }

    #[test]
    fn calibration_manifest_requires_known_case_kinds() {
        assert!(CALIBRATION_REQUIRED_KINDS.contains(&"positive"));
        assert!(CALIBRATION_REQUIRED_KINDS.contains(&"negative"));
        assert!(CALIBRATION_REQUIRED_KINDS.contains(&"false_positive_control"));
        assert!(!CALIBRATION_REQUIRED_KINDS.contains(&"aspirational"));
    }

    #[test]
    fn fixture_package_prefix_can_preserve_identity_fixture_package() {
        assert_eq!(
            fixture_package_prefix("raw_pointer_alignment_line_drift"),
            "raw-pointer-alignment"
        );
        assert_eq!(
            fixture_package_prefix("duplicate_raw_pointer_reads"),
            "duplicate-raw-pointer-reads"
        );
    }

    #[test]
    fn git_diff_shape_requires_file_headers() {
        assert!(looks_like_git_diff(
            "diff --git a/src/lib.rs b/src/lib.rs\n--- a/src/lib.rs\n+++ b/src/lib.rs\n"
        ));
        assert!(!looks_like_git_diff(
            "diff --git a/src/lib.rs b/src/lib.rs\n"
        ));
    }

    #[test]
    fn generated_artifact_detector_is_narrow() {
        assert!(is_forbidden_generated_path("target/debug/tool.exe"));
        assert!(is_forbidden_generated_path("reports/cards.sarif"));
        assert!(!is_forbidden_generated_path("Cargo.lock"));
        assert!(!is_forbidden_generated_path("docs/status/SUPPORT_TIERS.md"));
    }

    #[test]
    fn advisory_artifact_checker_accepts_expected_artifact_set() -> Result<(), String> {
        let dir = unique_temp_dir("unsafe-review-artifacts-ok")?;
        fs::create_dir_all(&dir).map_err(|err| format!("create temp dir failed: {err}"))?;
        write_valid_artifacts(&dir)?;

        let result = check_advisory_artifacts(&dir);

        fs::remove_dir_all(&dir).map_err(|err| format!("remove temp dir failed: {err}"))?;
        result
    }

    #[test]
    fn advisory_artifact_checker_rejects_missing_trust_boundary() -> Result<(), String> {
        let dir = unique_temp_dir("unsafe-review-artifacts-missing-boundary")?;
        fs::create_dir_all(&dir).map_err(|err| format!("create temp dir failed: {err}"))?;
        write_valid_artifacts(&dir)?;
        fs::write(dir.join("pr-summary.md"), "- Review cards: 1\n")
            .map_err(|err| format!("write pr summary failed: {err}"))?;

        let result = check_advisory_artifacts(&dir);

        fs::remove_dir_all(&dir).map_err(|err| format!("remove temp dir failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("static unsafe contract review")
        );
        Ok(())
    }

    #[test]
    fn advisory_artifact_checker_rejects_cards_json_without_trust_boundary() -> Result<(), String> {
        let dir = unique_temp_dir("unsafe-review-artifacts-missing-cards-boundary")?;
        fs::create_dir_all(&dir).map_err(|err| format!("create temp dir failed: {err}"))?;
        write_valid_artifacts(&dir)?;
        fs::write(
            dir.join("cards.json"),
            r#"{"tool":"unsafe-review","policy":"advisory","summary":{"cards":1},"cards":[{"id":"card-1"}]}"#,
        )
        .map_err(|err| format!("write cards failed: {err}"))?;

        let result = check_advisory_artifacts(&dir);

        fs::remove_dir_all(&dir).map_err(|err| format!("remove temp dir failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("cards.json is missing trust_boundary")
        );
        Ok(())
    }

    #[test]
    fn advisory_artifact_checker_rejects_unknown_projection_card_ids() -> Result<(), String> {
        let dir = unique_temp_dir("unsafe-review-artifacts-unknown-id")?;
        fs::create_dir_all(&dir).map_err(|err| format!("create temp dir failed: {err}"))?;
        write_valid_artifacts(&dir)?;
        fs::write(
            dir.join("comment-plan.json"),
            r#"{"mode":"plan_only","policy":"advisory","comments":[{"card_id":"missing"}],"trust_boundary":"static unsafe contract review, not a proof of memory safety, not UB-free status, and not a Miri result"}"#,
        )
        .map_err(|err| format!("write comment plan failed: {err}"))?;

        let result = check_advisory_artifacts(&dir);

        fs::remove_dir_all(&dir).map_err(|err| format!("remove temp dir failed: {err}"))?;
        assert!(result.err().unwrap_or_default().contains("unknown card id"));
        Ok(())
    }

    #[test]
    fn policy_ledger_accepts_empty_status_without_entries() -> Result<(), String> {
        let path = unique_temp_dir("unsafe-review-empty-ledger")?.with_extension("toml");
        fs::write(
            &path,
            r#"schema_version = "0.1"
policy = "unsafe-review-baseline"
status = "empty"
"#,
        )
        .map_err(|err| format!("write ledger failed: {err}"))?;

        let result = check_unsafe_review_ledger(&path, LedgerKind::Baseline);

        fs::remove_file(&path).map_err(|err| format!("remove ledger failed: {err}"))?;
        result
    }

    #[test]
    fn policy_ledger_requires_exact_counted_identity_metadata() -> Result<(), String> {
        let path = unique_temp_dir("unsafe-review-baseline-ledger")?.with_extension("toml");
        fs::write(
            &path,
            r#"schema_version = "0.1"
policy = "unsafe-review-baseline"
status = "active"

[[entries]]
card_id = "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1"
owner = "core/policy"
reason = "accepted current fixture debt"
evidence = "review-card fixture"
review_after = "2026-08-01"
"#,
        )
        .map_err(|err| format!("write ledger failed: {err}"))?;

        let result = check_unsafe_review_ledger(&path, LedgerKind::Baseline);

        fs::remove_file(&path).map_err(|err| format!("remove ledger failed: {err}"))?;
        result
    }

    #[test]
    fn suppression_ledger_requires_review_or_expiry_date() -> Result<(), String> {
        let path = unique_temp_dir("unsafe-review-suppression-ledger")?.with_extension("toml");
        fs::write(
            &path,
            r#"schema_version = "0.1"
policy = "unsafe-review-suppressions"
status = "active"

[[entries]]
card_id = "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1"
owner = "core/policy"
reason = "false positive under review"
evidence = "manual review"
"#,
        )
        .map_err(|err| format!("write ledger failed: {err}"))?;

        let result = check_unsafe_review_ledger(&path, LedgerKind::Suppression);

        fs::remove_file(&path).map_err(|err| format!("remove ledger failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("review_after or expires")
        );
        Ok(())
    }

    #[test]
    fn policy_ledger_rejects_uncounted_card_identity() -> Result<(), String> {
        let path = unique_temp_dir("unsafe-review-bad-identity-ledger")?.with_extension("toml");
        fs::write(
            &path,
            r#"schema_version = "0.1"
policy = "unsafe-review-baseline"
status = "active"

[[entries]]
card_id = "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment"
owner = "core/policy"
reason = "accepted current fixture debt"
evidence = "review-card fixture"
review_after = "2026-08-01"
"#,
        )
        .map_err(|err| format!("write ledger failed: {err}"))?;

        let result = check_unsafe_review_ledger(&path, LedgerKind::Baseline);

        fs::remove_file(&path).map_err(|err| format!("remove ledger failed: {err}"))?;
        assert!(result.err().unwrap_or_default().contains("exact counted"));
        Ok(())
    }

    fn write_valid_artifacts(dir: &Path) -> Result<(), String> {
        fs::write(
            dir.join("cards.json"),
            r#"{"tool":"unsafe-review","policy":"advisory","trust_boundary":"static unsafe contract review, not a proof of memory safety, not UB-free status, and not a Miri result","summary":{"cards":1},"cards":[{"id":"card-1"}]}"#,
        )
        .map_err(|err| format!("write cards failed: {err}"))?;
        fs::write(
            dir.join("pr-summary.md"),
            "- Review cards: 1\n\nThis artifact is static unsafe contract review, not a proof of memory safety, not UB-free status, and not a Miri result unless a witness receipt is attached.\n",
        )
        .map_err(|err| format!("write pr summary failed: {err}"))?;
        fs::write(
            dir.join("cards.sarif"),
            r#"{"version":"2.1.0","runs":[{"results":[{"properties":{"cardId":"card-1"}}],"properties":{"trustBoundary":"static unsafe contract review, not a proof of memory safety, not UB-free status, and not a Miri result"}}]}"#,
        )
        .map_err(|err| format!("write sarif failed: {err}"))?;
        fs::write(
            dir.join("comment-plan.json"),
            r#"{"mode":"plan_only","policy":"advisory","comments":[{"card_id":"card-1"}],"trust_boundary":"static unsafe contract review, not a proof of memory safety, not UB-free status, and not a Miri result"}"#,
        )
        .map_err(|err| format!("write comment plan failed: {err}"))?;
        Ok(())
    }

    fn unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("system clock before UNIX_EPOCH: {err}"))?
            .as_nanos();
        Ok(std::env::temp_dir().join(format!("{prefix}-{nanos}")))
    }
}
