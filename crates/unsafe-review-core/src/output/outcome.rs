use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const TRUST_BOUNDARY: &str = "Static unsafe contract review outcome only; this compares existing ReviewCard snapshots, not memory-safety proof, not UB-free status, and not witness execution.";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutcomeReport {
    pub schema_version: String,
    pub tool: String,
    pub mode: String,
    pub trust_boundary: String,
    pub before: OutcomeSnapshotSummary,
    pub after: OutcomeSnapshotSummary,
    pub summary: OutcomeSummary,
    pub cards: Vec<OutcomeCard>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutcomeSnapshotSummary {
    pub schema_version: String,
    pub cards: usize,
    pub open_actionable_gaps: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct OutcomeSummary {
    pub new: usize,
    pub resolved: usize,
    pub improved: usize,
    pub regressed: usize,
    pub unchanged: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutcomeCard {
    pub card_id: String,
    pub status: String,
    pub before: Option<OutcomeCardState>,
    pub after: Option<OutcomeCardState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OutcomeCardState {
    #[serde(rename = "class")]
    pub class_name: String,
    pub priority: String,
    pub missing_count: usize,
    pub missing: Vec<String>,
}

#[derive(Deserialize)]
struct Snapshot {
    schema_version: String,
    summary: SnapshotSummary,
    cards: Vec<SnapshotCard>,
}

#[derive(Deserialize)]
struct SnapshotSummary {
    cards: usize,
    open_actionable_gaps: usize,
}

#[derive(Clone, Deserialize)]
struct SnapshotCard {
    id: String,
    #[serde(rename = "class")]
    class_name: String,
    priority: String,
    #[serde(default)]
    missing: Vec<String>,
}

pub fn compare_json(before_json: &str, after_json: &str) -> Result<OutcomeReport, String> {
    let before = parse_snapshot(before_json, "before")?;
    let after = parse_snapshot(after_json, "after")?;
    compare_snapshots(before, after)
}

pub fn render_json(report: &OutcomeReport) -> String {
    match serde_json::to_string_pretty(report) {
        Ok(text) => text,
        Err(err) => format!("{{\n  \"error\": \"outcome serialization failed: {err}\"\n}}"),
    }
}

pub fn render_markdown(report: &OutcomeReport) -> String {
    let mut out = String::new();
    out.push_str("# unsafe-review outcome\n\n");
    out.push_str("Static comparison of two existing unsafe-review JSON snapshots.\n\n");
    out.push_str("## Summary\n\n");
    out.push_str("| New | Resolved | Improved | Regressed | Unchanged |\n");
    out.push_str("|---:|---:|---:|---:|---:|\n");
    out.push_str(&format!(
        "| {} | {} | {} | {} | {} |\n\n",
        report.summary.new,
        report.summary.resolved,
        report.summary.improved,
        report.summary.regressed,
        report.summary.unchanged
    ));
    out.push_str("## Card outcomes\n\n");
    if report.cards.is_empty() {
        out.push_str("No cards in either snapshot.\n\n");
    } else {
        out.push_str("| Status | Card | Before | After |\n");
        out.push_str("|---|---|---|---|\n");
        for card in &report.cards {
            out.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                card.status,
                card.card_id,
                markdown_state(card.before.as_ref()),
                markdown_state(card.after.as_ref())
            ));
        }
        out.push('\n');
    }
    out.push_str("## Trust boundary\n\n");
    out.push_str(&report.trust_boundary);
    out.push('\n');
    out
}

fn parse_snapshot(text: &str, label: &str) -> Result<Snapshot, String> {
    let snapshot: Snapshot = serde_json::from_str(text)
        .map_err(|err| format!("parse {label} unsafe-review JSON snapshot failed: {err}"))?;
    if snapshot.schema_version.trim().is_empty() {
        return Err(format!("{label} snapshot is missing `schema_version`"));
    }
    if snapshot.summary.cards != snapshot.cards.len() {
        return Err(format!(
            "{label} snapshot summary card count {} does not match {} card object(s)",
            snapshot.summary.cards,
            snapshot.cards.len()
        ));
    }
    Ok(snapshot)
}

fn compare_snapshots(before: Snapshot, after: Snapshot) -> Result<OutcomeReport, String> {
    let before_summary = OutcomeSnapshotSummary::from(&before);
    let after_summary = OutcomeSnapshotSummary::from(&after);
    let before_cards = cards_by_id(before.cards, "before")?;
    let after_cards = cards_by_id(after.cards, "after")?;
    let ids = before_cards
        .keys()
        .chain(after_cards.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut summary = OutcomeSummary::default();
    let mut cards = Vec::new();
    for id in ids {
        let before = before_cards.get(&id);
        let after = after_cards.get(&id);
        let status = outcome_status(before, after);
        match status {
            "new" => summary.new += 1,
            "resolved" => summary.resolved += 1,
            "improved" => summary.improved += 1,
            "regressed" => summary.regressed += 1,
            _ => summary.unchanged += 1,
        }
        cards.push(OutcomeCard {
            card_id: id,
            status: status.to_string(),
            before: before.map(OutcomeCardState::from),
            after: after.map(OutcomeCardState::from),
        });
    }
    Ok(OutcomeReport {
        schema_version: "0.1".to_string(),
        tool: "unsafe-review".to_string(),
        mode: "outcome".to_string(),
        trust_boundary: TRUST_BOUNDARY.to_string(),
        before: before_summary,
        after: after_summary,
        summary,
        cards,
    })
}

fn cards_by_id(
    cards: Vec<SnapshotCard>,
    label: &str,
) -> Result<BTreeMap<String, SnapshotCard>, String> {
    let mut by_id = BTreeMap::new();
    for card in cards {
        if by_id.insert(card.id.clone(), card).is_some() {
            return Err(format!("{label} snapshot contains duplicate card id"));
        }
    }
    Ok(by_id)
}

fn outcome_status(before: Option<&SnapshotCard>, after: Option<&SnapshotCard>) -> &'static str {
    match (before, after) {
        (None, Some(_)) => "new",
        (Some(_), None) => "resolved",
        (Some(before), Some(after)) => changed_status(before, after),
        (None, None) => "unchanged",
    }
}

fn changed_status(before: &SnapshotCard, after: &SnapshotCard) -> &'static str {
    let before_actionable = is_actionable_class(&before.class_name);
    let after_actionable = is_actionable_class(&after.class_name);
    if before_actionable && !after_actionable {
        return "improved";
    }
    if !before_actionable && after_actionable {
        return "regressed";
    }
    let before_missing = before.missing.len();
    let after_missing = after.missing.len();
    if after_missing < before_missing {
        "improved"
    } else if after_missing > before_missing {
        "regressed"
    } else {
        "unchanged"
    }
}

fn is_actionable_class(value: &str) -> bool {
    matches!(
        value,
        "guarded_unwitnessed"
            | "contract_missing"
            | "guard_missing"
            | "reachable_unwitnessed"
            | "unsafe_unreached"
            | "witness_mismatch"
            | "requires_loom"
            | "requires_sanitizer"
            | "requires_kani_or_crux"
            | "miri_unsupported"
            | "static_unknown"
    )
}

fn markdown_state(state: Option<&OutcomeCardState>) -> String {
    match state {
        Some(state) => format!(
            "`{}` / `{}` / {} missing",
            state.class_name, state.priority, state.missing_count
        ),
        None => "-".to_string(),
    }
}

impl From<&Snapshot> for OutcomeSnapshotSummary {
    fn from(snapshot: &Snapshot) -> Self {
        Self {
            schema_version: snapshot.schema_version.clone(),
            cards: snapshot.summary.cards,
            open_actionable_gaps: snapshot.summary.open_actionable_gaps,
        }
    }
}

impl From<&SnapshotCard> for OutcomeCardState {
    fn from(card: &SnapshotCard) -> Self {
        Self {
            class_name: card.class_name.clone(),
            priority: card.priority.clone(),
            missing_count: card.missing.len(),
            missing: card.missing.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_json_reports_new_resolved_improved_regressed_and_unchanged() -> Result<(), String> {
        let before = snapshot_json(&[
            card("UR-a-c1", "guard_missing", "high", &["guard", "witness"]),
            card("UR-b-c1", "guard_missing", "high", &["guard"]),
            card("UR-c-c1", "guarded_and_witnessed", "low", &[]),
            card("UR-d-c1", "guard_missing", "high", &["guard"]),
        ]);
        let after = snapshot_json(&[
            card("UR-a-c1", "guard_missing", "high", &["guard"]),
            card("UR-c-c1", "guard_missing", "high", &["guard"]),
            card("UR-d-c1", "guard_missing", "high", &["guard"]),
            card("UR-e-c1", "contract_missing", "high", &["contract"]),
        ]);

        let report = compare_json(&before, &after)?;

        assert_eq!(report.schema_version, "0.1");
        assert_eq!(report.mode, "outcome");
        assert_eq!(report.summary.new, 1);
        assert_eq!(report.summary.resolved, 1);
        assert_eq!(report.summary.improved, 1);
        assert_eq!(report.summary.regressed, 1);
        assert_eq!(report.summary.unchanged, 1);
        assert!(
            report
                .trust_boundary
                .contains("compares existing ReviewCard snapshots")
        );
        Ok(())
    }

    #[test]
    fn outcome_renderers_are_parseable_and_keep_trust_boundary() -> Result<(), String> {
        let before = snapshot_json(&[]);
        let after = snapshot_json(&[card("UR-new-c1", "guard_missing", "high", &["guard"])]);
        let report = compare_json(&before, &after)?;
        let json = render_json(&report);
        let value: serde_json::Value =
            serde_json::from_str(&json).map_err(|err| format!("parse JSON failed: {err}"))?;
        assert_eq!(value["mode"], "outcome");
        assert_eq!(value["summary"]["new"], 1);
        assert!(
            value["trust_boundary"]
                .as_str()
                .unwrap_or("")
                .contains("not memory-safety proof")
        );

        let markdown = render_markdown(&report);
        assert!(markdown.contains("# unsafe-review outcome"));
        assert!(markdown.contains("## Trust boundary"));
        assert!(markdown.contains("UR-new-c1"));
        Ok(())
    }

    #[test]
    fn outcome_rejects_duplicate_card_identity() {
        let before = snapshot_json(&[
            card("UR-dup-c1", "guard_missing", "high", &["guard"]),
            card("UR-dup-c1", "contract_missing", "high", &["contract"]),
        ]);
        let after = snapshot_json(&[]);

        assert!(
            compare_json(&before, &after)
                .err()
                .unwrap_or_default()
                .contains("duplicate card id")
        );
    }

    #[test]
    fn outcome_rejects_summary_card_count_mismatch() {
        let before = r#"{
  "schema_version": "0.1",
  "summary": {
    "cards": 2,
    "open_actionable_gaps": 0
  },
  "cards": []
}"#;
        let after = snapshot_json(&[]);

        assert!(
            compare_json(before, &after)
                .err()
                .unwrap_or_default()
                .contains("summary card count")
        );
    }

    fn snapshot_json(cards: &[String]) -> String {
        format!(
            r#"{{
  "schema_version": "0.1",
  "summary": {{
    "cards": {},
    "open_actionable_gaps": {}
  }},
  "cards": [
    {}
  ]
}}"#,
            cards.len(),
            cards.len(),
            cards.join(",\n    ")
        )
    }

    fn card(id: &str, class_name: &str, priority: &str, missing: &[&str]) -> String {
        let missing = missing
            .iter()
            .map(|item| format!(r#""{item}""#))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"{{
      "id": "{id}",
      "class": "{class_name}",
      "priority": "{priority}",
      "missing": [{missing}]
    }}"#
        )
    }
}
