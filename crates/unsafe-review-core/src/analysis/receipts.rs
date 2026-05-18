use crate::domain::{CardId, WitnessEvidence};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub(crate) struct ReceiptIndex {
    by_card_id: BTreeMap<String, WitnessEvidence>,
}

impl ReceiptIndex {
    pub(crate) fn load(root: &Path) -> Result<Self, String> {
        let dir = root.join(".unsafe-review").join("receipts");
        if !dir.is_dir() {
            return Ok(Self::default());
        }
        let mut by_card_id = BTreeMap::new();
        let entries =
            fs::read_dir(&dir).map_err(|err| format!("read {} failed: {err}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|err| format!("read_dir entry failed: {err}"))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let receipt = parse_receipt_file(&path)?;
            if by_card_id
                .insert(receipt.card_id.clone(), receipt.evidence)
                .is_some()
            {
                return Err(format!(
                    "{} imports duplicate receipt for card_id `{}`",
                    path.display(),
                    receipt.card_id
                ));
            }
        }
        Ok(Self { by_card_id })
    }

    pub(crate) fn evidence_for(&self, id: &CardId) -> Option<WitnessEvidence> {
        self.by_card_id.get(&id.0).cloned()
    }
}

#[derive(Deserialize)]
struct ReceiptFile {
    schema_version: String,
    card_id: String,
    tool: String,
    strength: String,
    summary: Option<String>,
    command: Option<String>,
    limitations: Option<Vec<String>>,
}

struct ParsedReceipt {
    card_id: String,
    evidence: WitnessEvidence,
}

fn parse_receipt_file(path: &Path) -> Result<ParsedReceipt, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let receipt: ReceiptFile = serde_json::from_str(&text)
        .map_err(|err| format!("{} is not valid receipt JSON: {err}", path.display()))?;
    validate_required(&receipt.schema_version, "schema_version", path)?;
    validate_required(&receipt.card_id, "card_id", path)?;
    validate_required(&receipt.tool, "tool", path)?;
    validate_strength(&receipt.strength, path)?;
    if !looks_like_counted_card_id(&receipt.card_id) {
        return Err(format!(
            "{} card_id must be an exact counted UR-* identity ending in -cN",
            path.display()
        ));
    }
    Ok(ParsedReceipt {
        card_id: receipt.card_id.clone(),
        evidence: WitnessEvidence::present(receipt_summary(&receipt)),
    })
}

fn validate_required(value: &str, key: &str, path: &Path) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{} `{key}` is required", path.display()))
    } else {
        Ok(())
    }
}

fn validate_strength(value: &str, path: &Path) -> Result<(), String> {
    match value {
        "configured" | "ran" | "test_targeted" | "site_reached" => Ok(()),
        other => Err(format!(
            "{} uses unknown receipt strength `{other}`",
            path.display()
        )),
    }
}

fn receipt_summary(receipt: &ReceiptFile) -> String {
    let mut summary = format!(
        "Imported {} receipt with `{}` strength",
        receipt.tool, receipt.strength
    );
    if let Some(detail) = receipt.summary.as_deref().filter(|value| !value.is_empty()) {
        summary.push_str(": ");
        summary.push_str(detail);
    }
    if let Some(command) = receipt.command.as_deref().filter(|value| !value.is_empty()) {
        summary.push_str("; command: ");
        summary.push_str(command);
    }
    if let Some(limitations) = &receipt.limitations
        && !limitations.is_empty()
    {
        summary.push_str("; limitations: ");
        summary.push_str(&limitations.join("; "));
    }
    summary
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn receipt_index_loads_exact_card_receipts() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-receipt-index")?;
        let receipts = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipts).map_err(|err| format!("create receipt dir failed: {err}"))?;
        let card_id =
            "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1";
        fs::write(
            receipts.join("miri.json"),
            format!(
                r#"{{
  "schema_version": "0.1",
  "card_id": "{card_id}",
  "tool": "miri",
  "strength": "ran",
  "summary": "focused witness passed",
  "command": "cargo +nightly miri test read_header",
  "limitations": ["fixture only"]
}}"#
            ),
        )
        .map_err(|err| format!("write receipt failed: {err}"))?;

        let index = ReceiptIndex::load(&root)?;

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp root failed: {err}"))?;
        let evidence = index
            .evidence_for(&CardId(card_id.to_string()))
            .ok_or_else(|| "receipt evidence missing".to_string())?;
        assert!(evidence.present);
        assert!(evidence.summary.contains("miri"));
        assert!(evidence.summary.contains("ran"));
        assert!(evidence.summary.contains("fixture only"));
        Ok(())
    }

    #[test]
    fn receipt_index_rejects_unknown_strength() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-bad-receipt")?;
        let receipts = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipts).map_err(|err| format!("create receipt dir failed: {err}"))?;
        fs::write(
            receipts.join("bad.json"),
            r#"{
  "schema_version": "0.1",
  "card_id": "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1",
  "tool": "miri",
  "strength": "proved"
}"#,
        )
        .map_err(|err| format!("write receipt failed: {err}"))?;

        let result = ReceiptIndex::load(&root);

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp root failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("unknown receipt strength")
        );
        Ok(())
    }

    #[test]
    fn receipt_index_rejects_uncounted_card_identity() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-uncounted-receipt")?;
        let receipts = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipts).map_err(|err| format!("create receipt dir failed: {err}"))?;
        fs::write(
            receipts.join("bad.json"),
            r#"{
  "schema_version": "0.1",
  "card_id": "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment",
  "tool": "miri",
  "strength": "ran"
}"#,
        )
        .map_err(|err| format!("write receipt failed: {err}"))?;

        let result = ReceiptIndex::load(&root);

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp root failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("exact counted UR-* identity")
        );
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
