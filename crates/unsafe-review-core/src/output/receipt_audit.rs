use crate::analysis::receipts::ReceiptAuditReport;

pub(crate) fn render_json(report: &ReceiptAuditReport) -> String {
    match serde_json::to_string_pretty(report) {
        Ok(text) => text,
        Err(err) => format!("{{\n  \"error\": \"receipt audit serialization failed: {err}\"\n}}"),
    }
}

pub(crate) fn render_markdown(report: &ReceiptAuditReport) -> String {
    let mut out = String::new();
    out.push_str("# unsafe-review receipt audit\n\n");
    out.push_str("Static audit of saved witness receipt metadata against current ReviewCards.\n\n");
    out.push_str("## Summary\n\n");
    out.push_str("| Receipts | Matched | Unmatched | Expired | Stale | Wrong identity | Wrong tool | Weak strength | Duplicate | Invalid |\n");
    out.push_str("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    out.push_str(&format!(
        "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n\n",
        report.summary.receipts,
        report.summary.matched,
        report.summary.unmatched,
        report.summary.expired,
        report.summary.stale,
        report.summary.wrong_identity,
        report.summary.wrong_tool,
        report.summary.weaker_than_required,
        report.summary.duplicate,
        report.summary.invalid
    ));
    out.push_str("## Receipts\n\n");
    if report.receipts.is_empty() {
        out.push_str("No receipt files found.\n\n");
    } else {
        out.push_str("| Status | Receipt | Card | Tool | Strength | Issues |\n");
        out.push_str("|---|---|---|---|---|---|\n");
        for receipt in &report.receipts {
            out.push_str(&format!(
                "| {} | `{}` | {} | {} | {} | {} |\n",
                receipt.statuses.join(", "),
                receipt.path,
                optional_code(receipt.card_id.as_deref()),
                optional_code(receipt.receipt_tool.as_deref()),
                optional_code(receipt.strength.as_deref()),
                if receipt.issues.is_empty() {
                    "-".to_string()
                } else {
                    receipt.issues.join("; ")
                }
            ));
        }
        out.push('\n');
    }
    out.push_str("## Trust boundary\n\n");
    out.push_str(&report.trust_boundary);
    out.push('\n');
    out
}

fn optional_code(value: Option<&str>) -> String {
    match value {
        Some(value) if !value.is_empty() => format!("`{value}`"),
        _ => "-".to_string(),
    }
}
