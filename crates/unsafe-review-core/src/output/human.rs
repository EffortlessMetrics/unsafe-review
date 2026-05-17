use crate::api::AnalyzeOutput;
use crate::util::path_display;

pub(crate) fn render(output: &AnalyzeOutput) -> String {
    let mut out = String::new();
    out.push_str("unsafe-review\n");
    out.push_str(&format!(
        "scope: {:?}, mode: {}, policy: {}\n",
        output.scope,
        output.mode.as_str(),
        output.policy.as_str()
    ));
    out.push_str(&format!(
        "cards: {}, open gaps: {}, contract_missing: {}, guard_missing: {}, witness gaps: {}\n\n",
        output.summary.cards,
        output.summary.open_actionable_gaps,
        output.summary.contract_missing,
        output.summary.guard_missing,
        output.summary.guarded_unwitnessed
    ));

    if output.cards.is_empty() {
        out.push_str("No unsafe-review cards found.\n");
        return out;
    }

    for card in &output.cards {
        out.push_str(&format!(
            "{} {}:{}\n",
            card.class.as_str().to_uppercase(),
            path_display(&card.site.location.file),
            card.site.location.line
        ));
        out.push_str(&format!("  id: {}\n", card.id));
        out.push_str(&format!("  operation: {}\n", card.operation.expression));
        out.push_str(&format!("  contract: {}\n", card.contract.summary));
        out.push_str(&format!("  discharge: {}\n", card.discharge.summary));
        out.push_str(&format!("  reach: {}\n", card.reach.summary));
        out.push_str("  missing:\n");
        for missing in &card.missing {
            out.push_str(&format!("    - {}\n", missing.message));
        }
        out.push_str(&format!("  next: {}\n", card.next_action.summary));
        if !card.next_action.verify_commands.is_empty() {
            out.push_str("  verify:\n");
            for cmd in &card.next_action.verify_commands {
                out.push_str(&format!("    {}\n", cmd));
            }
        }
        out.push('\n');
    }

    out.push_str("Trust boundary: static unsafe contract review; not a proof of memory safety and not a Miri result unless a witness receipt is attached.\n");
    out
}
