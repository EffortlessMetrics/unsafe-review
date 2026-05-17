use crate::api::AnalyzeOutput;
use crate::domain::ReviewCard;
use crate::util::path_display;

pub(crate) fn render(output: &AnalyzeOutput) -> String {
    let mut out = String::new();
    out.push_str("# unsafe-review\n\n");
    out.push_str(&format!(
        "{} changed/repo unsafe seam card(s) found.\n\n",
        output.summary.cards
    ));
    out.push_str("## Recommended next action\n\n");
    if let Some(card) = output.cards.first() {
        out.push_str(&card.next_action.summary);
        out.push_str("\n\n");
        if let Some(cmd) = card.next_action.verify_commands.first() {
            out.push_str("```bash\n");
            out.push_str(cmd);
            out.push_str("\n```\n\n");
        }
    } else {
        out.push_str("No actionable unsafe-review cards found.\n\n");
    }
    out.push_str("## Cards\n\n");
    out.push_str("| ID | Class | Hazard | Missing | Route |\n");
    out.push_str("|---|---|---|---|---|\n");
    for card in &output.cards {
        let hazard = card.hazards.first().map_or("unknown", |h| h.as_str());
        let missing = card.missing.first().map_or("", |m| m.kind.as_str());
        let route = card.routes.first().map_or("human", |r| r.kind.as_str());
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            card.id,
            card.class.as_str(),
            hazard,
            missing,
            route
        ));
    }
    out.push_str("\n## Trust boundary\n\n");
    out.push_str("This is static unsafe contract review. It is not a proof of memory safety and not a Miri result unless a witness receipt is attached.\n");
    out
}

pub(crate) fn render_card_detail(card: &ReviewCard) -> String {
    let mut out = String::new();
    out.push_str(&format!("# unsafe-review card `{}`\n\n", card.id));
    out.push_str(&format!("**Class:** `{}`\n\n", card.class.as_str()));
    out.push_str(&format!(
        "**Location:** {}:{}\n\n",
        path_display(&card.site.location.file),
        card.site.location.line
    ));
    out.push_str(&format!(
        "**Operation:** `{}`\n\n",
        card.operation.expression
    ));
    out.push_str("## Required safety conditions\n\n");
    for obligation in &card.obligations {
        out.push_str(&format!("- {}\n", obligation.description));
    }
    out.push_str("\n## Evidence\n\n");
    out.push_str(&format!("- Contract: {}\n", card.contract.summary));
    out.push_str(&format!("- Discharge: {}\n", card.discharge.summary));
    out.push_str(&format!("- Reach: {}\n", card.reach.summary));
    out.push_str(&format!("- Witness: {}\n", card.witness.summary));
    out.push_str("\n## Next action\n\n");
    out.push_str(&card.next_action.summary);
    out.push('\n');
    out
}
