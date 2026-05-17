use crate::api::AnalyzeOutput;
use crate::domain::ReviewCard;
use crate::util::{json_escape, path_display};

pub(crate) fn render(output: &AnalyzeOutput) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    field(&mut out, 1, "schema_version", &output.schema_version, true);
    field(&mut out, 1, "tool", &output.tool, true);
    field(&mut out, 1, "scope", scope_str(output), true);
    field(&mut out, 1, "mode", output.mode.as_str(), true);
    field(&mut out, 1, "policy", output.policy.as_str(), true);
    field(&mut out, 1, "root", &path_display(&output.root), true);
    out.push_str("  \"summary\": ");
    summary(&mut out, output);
    out.push_str(",\n  \"cards\": [");
    for (idx, card) in output.cards.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('\n');
        card_json(&mut out, card, 2);
    }
    if !output.cards.is_empty() {
        out.push('\n');
    }
    out.push_str("  ]\n}");
    out
}

pub(crate) fn render_agent_packet(card: &ReviewCard) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    field(&mut out, 1, "schema_version", "0.1", true);
    field(&mut out, 1, "tool", "unsafe-review", true);
    field(&mut out, 1, "card_id", &card.id.0, true);
    field(&mut out, 1, "task", &card.next_action.summary, true);
    out.push_str("  \"context\": {\n");
    field(
        &mut out,
        2,
        "file",
        &path_display(&card.site.location.file),
        true,
    );
    number_field(&mut out, 2, "line", card.site.location.line, true);
    field(&mut out, 2, "operation", &card.operation.expression, false);
    out.push_str("\n  },\n");
    string_array(
        &mut out,
        1,
        "required_safety_conditions",
        &card
            .obligations
            .iter()
            .map(|o| o.description.clone())
            .collect::<Vec<_>>(),
        true,
    );
    string_array(
        &mut out,
        1,
        "missing",
        &card
            .missing
            .iter()
            .map(|m| m.message.clone())
            .collect::<Vec<_>>(),
        true,
    );
    string_array(
        &mut out,
        1,
        "allowed_repairs",
        std::slice::from_ref(&card.next_action.summary),
        true,
    );
    string_array(
        &mut out,
        1,
        "verify_commands",
        &card.next_action.verify_commands,
        true,
    );
    string_array(
        &mut out,
        1,
        "do_not_do",
        &[
            "do not widen unsafe code without reducing the missing evidence".to_string(),
            "do not add a broad suppression".to_string(),
            "do not claim Miri proof unless the witness command is run and attached".to_string(),
        ],
        true,
    );
    string_array(
        &mut out,
        1,
        "stop_conditions",
        &[
            "the missing evidence is present or explicitly waived with owner and expiry"
                .to_string(),
            "the focused test or witness command has been run or marked unavailable".to_string(),
            "no unrelated unsafe code was changed".to_string(),
        ],
        false,
    );
    out.push_str("\n}\n");
    out
}

fn scope_str(output: &AnalyzeOutput) -> &'static str {
    match output.scope {
        crate::api::Scope::Diff => "diff",
        crate::api::Scope::Repo => "repo",
    }
}

fn summary(out: &mut String, output: &AnalyzeOutput) {
    out.push_str("{\n");
    number_field(out, 2, "rust_files", output.summary.rust_files, true);
    number_field(
        out,
        2,
        "changed_rust_files",
        output.summary.changed_rust_files,
        true,
    );
    number_field(out, 2, "unsafe_sites", output.summary.unsafe_sites, true);
    number_field(out, 2, "cards", output.summary.cards, true);
    number_field(
        out,
        2,
        "open_actionable_gaps",
        output.summary.open_actionable_gaps,
        true,
    );
    number_field(
        out,
        2,
        "contract_missing",
        output.summary.contract_missing,
        true,
    );
    number_field(out, 2, "guard_missing", output.summary.guard_missing, true);
    number_field(
        out,
        2,
        "guarded_unwitnessed",
        output.summary.guarded_unwitnessed,
        true,
    );
    number_field(
        out,
        2,
        "unsafe_unreached",
        output.summary.unsafe_unreached,
        true,
    );
    number_field(out, 2, "requires_loom", output.summary.requires_loom, true);
    number_field(
        out,
        2,
        "miri_unsupported",
        output.summary.miri_unsupported,
        true,
    );
    number_field(
        out,
        2,
        "static_unknown",
        output.summary.static_unknown,
        false,
    );
    out.push_str("\n  }");
}

fn card_json(out: &mut String, card: &ReviewCard, indent: usize) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}{{\n", pad));
    field(out, indent + 1, "id", &card.id.0, true);
    field(out, indent + 1, "class", card.class.as_str(), true);
    field(out, indent + 1, "priority", card.priority.as_str(), true);
    field(
        out,
        indent + 1,
        "confidence",
        card.confidence.as_str(),
        true,
    );
    out.push_str(&format!("{}  \"site\": {{\n", pad));
    field(
        out,
        indent + 2,
        "file",
        &path_display(&card.site.location.file),
        true,
    );
    number_field(out, indent + 2, "line", card.site.location.line, true);
    number_field(out, indent + 2, "column", card.site.location.column, true);
    field(out, indent + 2, "kind", card.site.kind.as_str(), true);
    field(
        out,
        indent + 2,
        "owner",
        card.site.owner.as_deref().unwrap_or(""),
        true,
    );
    field(out, indent + 2, "snippet", &card.site.snippet, false);
    out.push_str(&format!("\n{}  }},\n", pad));
    field(
        out,
        indent + 1,
        "operation_family",
        card.operation.family.as_str(),
        true,
    );
    string_array(
        out,
        indent + 1,
        "hazards",
        &card
            .hazards
            .iter()
            .map(|h| h.as_str().to_string())
            .collect::<Vec<_>>(),
        true,
    );
    string_array(
        out,
        indent + 1,
        "obligations",
        &card
            .obligations
            .iter()
            .map(|o| o.description.clone())
            .collect::<Vec<_>>(),
        true,
    );
    field(out, indent + 1, "contract", &card.contract.summary, true);
    field(out, indent + 1, "discharge", &card.discharge.summary, true);
    field(out, indent + 1, "reach", &card.reach.summary, true);
    field(out, indent + 1, "witness", &card.witness.summary, true);
    string_array(
        out,
        indent + 1,
        "missing",
        &card
            .missing
            .iter()
            .map(|m| m.message.clone())
            .collect::<Vec<_>>(),
        true,
    );
    string_array(
        out,
        indent + 1,
        "verify_commands",
        &card.next_action.verify_commands,
        false,
    );
    out.push_str(&format!("\n{}}}", pad));
}

fn field(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": \"{}\"{}\n",
        pad,
        key,
        json_escape(value),
        if comma { "," } else { "" }
    ));
}

fn number_field(out: &mut String, indent: usize, key: &str, value: usize, comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!(
        "{}\"{}\": {}{}\n",
        pad,
        key,
        value,
        if comma { "," } else { "" }
    ));
}

fn string_array(out: &mut String, indent: usize, key: &str, values: &[String], comma: bool) {
    let pad = "  ".repeat(indent);
    out.push_str(&format!("{}\"{}\": [", pad, key));
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("\"{}\"", json_escape(value)));
    }
    out.push_str(&format!("]{}\n", if comma { "," } else { "" }));
}
