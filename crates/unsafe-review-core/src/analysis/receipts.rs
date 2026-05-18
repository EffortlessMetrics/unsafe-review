use crate::domain::{CardId, WitnessEvidence, WitnessReceipt};
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

struct ParsedReceipt {
    card_id: String,
    evidence: WitnessEvidence,
}

fn parse_receipt_file(path: &Path) -> Result<ParsedReceipt, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let receipt: WitnessReceipt = serde_json::from_str(&text)
        .map_err(|err| format!("{} is not valid receipt JSON: {err}", path.display()))?;
    receipt
        .validate()
        .map_err(|err| format!("{} {err}", path.display()))?;
    Ok(ParsedReceipt {
        card_id: receipt.card_id.clone(),
        evidence: WitnessEvidence::present(receipt.evidence_summary()),
    })
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
  "author": "core/fixtures",
  "recorded_at": "2026-05-18T00:00:00Z",
  "expires_at": "2026-08-18",
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
        assert!(evidence.summary.contains("core/fixtures"));
        assert!(evidence.summary.contains("2026-08-18"));
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
    fn receipt_index_rejects_unknown_tool() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-bad-tool-receipt")?;
        let receipts = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipts).map_err(|err| format!("create receipt dir failed: {err}"))?;
        fs::write(
            receipts.join("bad.json"),
            r#"{
  "schema_version": "0.1",
  "card_id": "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1",
  "tool": "proof-bot",
  "strength": "ran",
  "author": "core/fixtures",
  "recorded_at": "2026-05-18T00:00:00Z",
  "expires_at": "2026-08-18"
}"#,
        )
        .map_err(|err| format!("write receipt failed: {err}"))?;

        let result = ReceiptIndex::load(&root);

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp root failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("unknown receipt tool")
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

    #[test]
    fn receipt_index_rejects_missing_author() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-missing-author-receipt")?;
        let receipts = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipts).map_err(|err| format!("create receipt dir failed: {err}"))?;
        fs::write(
            receipts.join("bad.json"),
            r#"{
  "schema_version": "0.1",
  "card_id": "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1",
  "tool": "miri",
  "strength": "ran",
  "recorded_at": "2026-05-18T00:00:00Z",
  "expires_at": "2026-08-18"
}"#,
        )
        .map_err(|err| format!("write receipt failed: {err}"))?;

        let result = ReceiptIndex::load(&root);

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp root failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("`author` is required")
        );
        Ok(())
    }

    #[test]
    fn receipt_index_rejects_invalid_recorded_at_timestamp() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-invalid-recorded-at-receipt")?;
        let receipts = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipts).map_err(|err| format!("create receipt dir failed: {err}"))?;
        fs::write(
            receipts.join("bad.json"),
            r#"{
  "schema_version": "0.1",
  "card_id": "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1",
  "tool": "miri",
  "strength": "ran",
  "author": "core/fixtures",
  "recorded_at": "2026-05-18",
  "expires_at": "2026-08-18"
}"#,
        )
        .map_err(|err| format!("write receipt failed: {err}"))?;

        let result = ReceiptIndex::load(&root);

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp root failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("UTC timestamp format")
        );
        Ok(())
    }

    #[test]
    fn receipt_index_rejects_expiry_before_recorded_date() -> Result<(), String> {
        let root = unique_temp_dir("unsafe-review-expired-receipt")?;
        let receipts = root.join(".unsafe-review").join("receipts");
        fs::create_dir_all(&receipts).map_err(|err| format!("create receipt dir failed: {err}"))?;
        fs::write(
            receipts.join("bad.json"),
            r#"{
  "schema_version": "0.1",
  "card_id": "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1",
  "tool": "miri",
  "strength": "ran",
  "author": "core/fixtures",
  "recorded_at": "2026-05-18T00:00:00Z",
  "expires_at": "2026-05-17"
}"#,
        )
        .map_err(|err| format!("write receipt failed: {err}"))?;

        let result = ReceiptIndex::load(&root);

        fs::remove_dir_all(&root).map_err(|err| format!("remove temp root failed: {err}"))?;
        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("on or after the `recorded_at` date")
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
