use crate::api::{AnalyzeOutput, Scope, Summary};
use crate::domain::{EvidenceState, ObligationEvidence, ReviewCard};
use crate::util::path_display;
use serde::Serialize;

pub(crate) fn render(output: &AnalyzeOutput) -> String {
    render_pretty(&JsonAnalyzeOutput::from(output))
}

pub(crate) fn render_agent_packet(card: &ReviewCard) -> String {
    render_pretty(&JsonAgentPacket::from(card))
}

fn render_pretty(value: &impl Serialize) -> String {
    match serde_json::to_string_pretty(value) {
        Ok(text) => text,
        Err(err) => format!("{{\n  \"error\": \"json serialization failed: {err}\"\n}}"),
    }
}

#[derive(Serialize)]
struct JsonAnalyzeOutput<'a> {
    schema_version: &'a str,
    tool: &'a str,
    scope: &'static str,
    mode: &'static str,
    policy: &'static str,
    root: String,
    summary: JsonSummary,
    cards: Vec<JsonCard<'a>>,
}

impl<'a> From<&'a AnalyzeOutput> for JsonAnalyzeOutput<'a> {
    fn from(output: &'a AnalyzeOutput) -> Self {
        Self {
            schema_version: &output.schema_version,
            tool: &output.tool,
            scope: scope_str(output),
            mode: output.mode.as_str(),
            policy: output.policy.as_str(),
            root: path_display(&output.root),
            summary: JsonSummary::from(&output.summary),
            cards: output.cards.iter().map(JsonCard::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct JsonSummary {
    rust_files: usize,
    changed_rust_files: usize,
    unsafe_sites: usize,
    cards: usize,
    open_actionable_gaps: usize,
    contract_missing: usize,
    guard_missing: usize,
    guarded_unwitnessed: usize,
    unsafe_unreached: usize,
    requires_loom: usize,
    miri_unsupported: usize,
    static_unknown: usize,
}

impl From<&Summary> for JsonSummary {
    fn from(summary: &Summary) -> Self {
        Self {
            rust_files: summary.rust_files,
            changed_rust_files: summary.changed_rust_files,
            unsafe_sites: summary.unsafe_sites,
            cards: summary.cards,
            open_actionable_gaps: summary.open_actionable_gaps,
            contract_missing: summary.contract_missing,
            guard_missing: summary.guard_missing,
            guarded_unwitnessed: summary.guarded_unwitnessed,
            unsafe_unreached: summary.unsafe_unreached,
            requires_loom: summary.requires_loom,
            miri_unsupported: summary.miri_unsupported,
            static_unknown: summary.static_unknown,
        }
    }
}

#[derive(Serialize)]
struct JsonCard<'a> {
    id: &'a str,
    #[serde(rename = "class")]
    class_name: &'static str,
    priority: &'static str,
    confidence: &'static str,
    site: JsonSite<'a>,
    operation_family: &'static str,
    hazards: Vec<&'static str>,
    obligations: Vec<&'a str>,
    obligation_evidence: Vec<JsonObligationEvidence<'a>>,
    contract: &'a str,
    discharge: &'a str,
    reach: &'a str,
    witness: &'a str,
    missing: Vec<&'a str>,
    verify_commands: &'a [String],
}

impl<'a> From<&'a ReviewCard> for JsonCard<'a> {
    fn from(card: &'a ReviewCard) -> Self {
        Self {
            id: &card.id.0,
            class_name: card.class.as_str(),
            priority: card.priority.as_str(),
            confidence: card.confidence.as_str(),
            site: JsonSite::from(card),
            operation_family: card.operation.family.as_str(),
            hazards: card.hazards.iter().map(|hazard| hazard.as_str()).collect(),
            obligations: card
                .obligations
                .iter()
                .map(|obligation| obligation.description.as_str())
                .collect(),
            obligation_evidence: card
                .obligation_evidence
                .iter()
                .map(JsonObligationEvidence::from)
                .collect(),
            contract: &card.contract.summary,
            discharge: &card.discharge.summary,
            reach: &card.reach.summary,
            witness: &card.witness.summary,
            missing: card
                .missing
                .iter()
                .map(|missing| missing.message.as_str())
                .collect(),
            verify_commands: &card.next_action.verify_commands,
        }
    }
}

#[derive(Serialize)]
struct JsonObligationEvidence<'a> {
    key: &'a str,
    description: &'a str,
    contract: JsonEvidenceState<'a>,
    discharge: JsonEvidenceState<'a>,
    reach: JsonEvidenceState<'a>,
    witness: JsonEvidenceState<'a>,
}

impl<'a> From<&'a ObligationEvidence> for JsonObligationEvidence<'a> {
    fn from(evidence: &'a ObligationEvidence) -> Self {
        Self {
            key: &evidence.obligation.key,
            description: &evidence.obligation.description,
            contract: JsonEvidenceState::from(&evidence.contract),
            discharge: JsonEvidenceState::from(&evidence.discharge),
            reach: JsonEvidenceState::from(&evidence.reach),
            witness: JsonEvidenceState::from(&evidence.witness),
        }
    }
}

#[derive(Serialize)]
struct JsonEvidenceState<'a> {
    present: bool,
    state: &'a str,
    summary: &'a str,
}

impl<'a> From<&'a EvidenceState> for JsonEvidenceState<'a> {
    fn from(state: &'a EvidenceState) -> Self {
        Self {
            present: state.present,
            state: &state.state,
            summary: &state.summary,
        }
    }
}

#[derive(Serialize)]
struct JsonSite<'a> {
    file: String,
    line: usize,
    column: usize,
    kind: &'static str,
    owner: &'a str,
    snippet: &'a str,
}

impl<'a> From<&'a ReviewCard> for JsonSite<'a> {
    fn from(card: &'a ReviewCard) -> Self {
        Self {
            file: path_display(&card.site.location.file),
            line: card.site.location.line,
            column: card.site.location.column,
            kind: card.site.kind.as_str(),
            owner: card.site.owner.as_deref().unwrap_or(""),
            snippet: &card.site.snippet,
        }
    }
}

#[derive(Serialize)]
struct JsonAgentPacket<'a> {
    schema_version: &'static str,
    tool: &'static str,
    card_id: &'a str,
    task: &'a str,
    context: JsonAgentContext<'a>,
    required_safety_conditions: Vec<&'a str>,
    obligation_evidence: Vec<JsonObligationEvidence<'a>>,
    missing: Vec<&'a str>,
    allowed_repairs: Vec<&'a str>,
    verify_commands: &'a [String],
    do_not_do: &'static [&'static str],
    stop_conditions: &'static [&'static str],
}

impl<'a> From<&'a ReviewCard> for JsonAgentPacket<'a> {
    fn from(card: &'a ReviewCard) -> Self {
        Self {
            schema_version: "0.1",
            tool: "unsafe-review",
            card_id: &card.id.0,
            task: &card.next_action.summary,
            context: JsonAgentContext::from(card),
            required_safety_conditions: card
                .obligations
                .iter()
                .map(|obligation| obligation.description.as_str())
                .collect(),
            obligation_evidence: card
                .obligation_evidence
                .iter()
                .map(JsonObligationEvidence::from)
                .collect(),
            missing: card
                .missing
                .iter()
                .map(|missing| missing.message.as_str())
                .collect(),
            allowed_repairs: vec![card.next_action.summary.as_str()],
            verify_commands: &card.next_action.verify_commands,
            do_not_do: &[
                "do not widen unsafe code without reducing the missing evidence",
                "do not add a broad suppression",
                "do not claim Miri proof unless the witness command is run and attached",
            ],
            stop_conditions: &[
                "the missing evidence is present or explicitly waived with owner and expiry",
                "the focused test or witness command has been run or marked unavailable",
                "no unrelated unsafe code was changed",
            ],
        }
    }
}

#[derive(Serialize)]
struct JsonAgentContext<'a> {
    file: String,
    line: usize,
    operation: &'a str,
}

impl<'a> From<&'a ReviewCard> for JsonAgentContext<'a> {
    fn from(card: &'a ReviewCard) -> Self {
        Self {
            file: path_display(&card.site.location.file),
            line: card.site.location.line,
            operation: &card.operation.expression,
        }
    }
}

fn scope_str(output: &AnalyzeOutput) -> &'static str {
    match output.scope {
        Scope::Diff => "diff",
        Scope::Repo => "repo",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{AnalysisMode, AnalyzeInput, DiffSource, PolicyMode, analyze};
    use std::fs;
    use std::path::PathBuf;

    const FIXTURE_GOLDENS: &[&str] = &[
        "raw_pointer_alignment",
        "safe_code_no_cards",
        "public_unsafe_fn_missing_safety",
        "split_public_unsafe_fn_missing_safety",
        "split_unsafe_block",
        "raw_pointer_deref",
        "safe_reference_deref_no_cards",
        "split_raw_pointer_read_call",
        "maybeuninit_assume_init",
    ];

    #[test]
    fn rendered_analysis_json_is_parseable_and_keeps_card_contract() -> Result<(), String> {
        let output = fixture_output("raw_pointer_alignment")?;
        let value = parse_json(&render(&output))?;

        assert_eq!(value["schema_version"], "0.1");
        assert_eq!(value["tool"], "unsafe-review");
        assert_eq!(value["scope"], "diff");
        assert_eq!(value["summary"]["cards"], 1);
        assert_eq!(value["cards"][0]["class"], "guard_missing");
        assert_eq!(value["cards"][0]["site"]["file"], "src/lib.rs");
        assert_eq!(value["cards"][0]["operation_family"], "raw_pointer_read");
        assert!(value["cards"][0]["obligation_evidence"].is_array());
        assert!(value["cards"][0]["verify_commands"].is_array());
        Ok(())
    }

    #[test]
    fn rendered_agent_packet_json_is_parseable_and_bounded() -> Result<(), String> {
        let output = fixture_output("raw_pointer_alignment")?;
        let Some(card) = output.cards.first() else {
            return Err("fixture should emit one card".to_string());
        };
        let value = parse_json(&render_agent_packet(card))?;

        assert_eq!(value["schema_version"], "0.1");
        assert_eq!(value["tool"], "unsafe-review");
        assert_eq!(value["card_id"], card.id.0);
        assert!(value["required_safety_conditions"].is_array());
        assert!(value["obligation_evidence"].is_array());
        assert!(value["allowed_repairs"].is_array());
        assert!(value["do_not_do"].is_array());
        assert!(value["stop_conditions"].is_array());
        Ok(())
    }

    #[test]
    fn fixture_card_goldens_match_rendered_json() -> Result<(), String> {
        for fixture in FIXTURE_GOLDENS {
            let output = fixture_output(fixture)?;
            let actual = parse_json(&render(&output))?;
            let expected = fixture_expected_cards(fixture)?;
            let Some(actual_cards) = actual.get("cards") else {
                return Err(format!("{fixture} JSON output is missing `cards`"));
            };
            if actual_cards != &expected {
                return Err(format!(
                    "{fixture} card JSON drifted\nexpected:\n{}\nactual:\n{}",
                    pretty_json(&expected),
                    pretty_json(actual_cards)
                ));
            }
        }
        Ok(())
    }

    fn fixture_output(name: &str) -> Result<AnalyzeOutput, String> {
        let root = fixture_root(name);
        analyze(AnalyzeInput {
            root: root.clone(),
            scope: Scope::Diff,
            diff: DiffSource::File(root.join("change.diff")),
            mode: AnalysisMode::Draft,
            policy: PolicyMode::Advisory,
            include_unchanged_tests: true,
            max_cards: None,
        })
    }

    fn fixture_expected_cards(name: &str) -> Result<serde_json::Value, String> {
        let path = fixture_root(name).join("expected.cards.json");
        let text = fs::read_to_string(&path)
            .map_err(|err| format!("read {} failed: {err}", path.display()))?;
        parse_json(&text)
    }

    fn fixture_root(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(name)
    }

    fn parse_json(text: &str) -> Result<serde_json::Value, String> {
        serde_json::from_str(text).map_err(|err| format!("JSON parse failed: {err}"))
    }

    fn pretty_json(value: &serde_json::Value) -> String {
        match serde_json::to_string_pretty(value) {
            Ok(text) => text,
            Err(err) => format!("<failed to render JSON: {err}>"),
        }
    }
}
