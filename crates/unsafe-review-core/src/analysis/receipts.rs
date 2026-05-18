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
    author: Option<String>,
    recorded_at: Option<String>,
    expires_at: Option<String>,
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
    validate_tool(&receipt.tool, path)?;
    validate_strength(&receipt.strength, path)?;
    if !looks_like_counted_card_id(&receipt.card_id) {
        return Err(format!(
            "{} card_id must be an exact counted UR-* identity ending in -cN",
            path.display()
        ));
    }
    validate_required_option(&receipt.author, "author", path)?;
    let recorded_at = validate_required_option(&receipt.recorded_at, "recorded_at", path)?;
    let expires_at = validate_required_option(&receipt.expires_at, "expires_at", path)?;
    validate_utc_timestamp(recorded_at, "recorded_at", path)?;
    validate_date(expires_at, "expires_at", path)?;
    if expires_at < &recorded_at[..10] {
        return Err(format!(
            "{} `expires_at` must be on or after the `recorded_at` date",
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

fn validate_required_option<'a>(
    value: &'a Option<String>,
    key: &str,
    path: &Path,
) -> Result<&'a str, String> {
    let Some(value) = value.as_deref() else {
        return Err(format!("{} `{key}` is required", path.display()));
    };
    validate_required(value, key, path)?;
    Ok(value)
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

fn validate_tool(value: &str, path: &Path) -> Result<(), String> {
    match value {
        "miri" | "cargo-careful" | "asan" | "msan" | "tsan" | "lsan" | "loom" | "shuttle"
        | "kani" | "crux" | "human-deep-review" | "unsupported" => Ok(()),
        other => Err(format!(
            "{} uses unknown receipt tool `{other}`",
            path.display()
        )),
    }
}

fn validate_utc_timestamp(value: &str, key: &str, path: &Path) -> Result<(), String> {
    let bytes = value.as_bytes();
    let valid_shape = bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18]
            .iter()
            .all(|index| bytes[*index].is_ascii_digit());
    if !valid_shape {
        return Err(format!(
            "{} `{key}` must use UTC timestamp format YYYY-MM-DDTHH:MM:SSZ",
            path.display()
        ));
    }
    validate_date(&value[..10], key, path)?;
    validate_range(decimal_at(value, 11, 2), 0, 23, key, path)?;
    validate_range(decimal_at(value, 14, 2), 0, 59, key, path)?;
    validate_range(decimal_at(value, 17, 2), 0, 59, key, path)
}

fn validate_date(value: &str, key: &str, path: &Path) -> Result<(), String> {
    let bytes = value.as_bytes();
    let valid_shape = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && [0, 1, 2, 3, 5, 6, 8, 9]
            .iter()
            .all(|index| bytes[*index].is_ascii_digit());
    if !valid_shape {
        return Err(format!(
            "{} `{key}` must use date format YYYY-MM-DD",
            path.display()
        ));
    }
    validate_range(decimal_at(value, 0, 4), 1, 9999, key, path)?;
    validate_range(decimal_at(value, 5, 2), 1, 12, key, path)?;
    validate_range(decimal_at(value, 8, 2), 1, 31, key, path)
}

fn decimal_at(value: &str, start: usize, len: usize) -> Option<u32> {
    value.get(start..start + len)?.parse().ok()
}

fn validate_range(
    value: Option<u32>,
    min: u32,
    max: u32,
    key: &str,
    path: &Path,
) -> Result<(), String> {
    let Some(value) = value else {
        return Err(format!(
            "{} `{key}` contains an invalid number",
            path.display()
        ));
    };
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("{} `{key}` is out of range", path.display()))
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
    if let Some(author) = receipt.author.as_deref().filter(|value| !value.is_empty()) {
        summary.push_str("; author: ");
        summary.push_str(author);
    }
    if let Some(recorded_at) = receipt
        .recorded_at
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        summary.push_str("; recorded_at: ");
        summary.push_str(recorded_at);
    }
    if let Some(expires_at) = receipt
        .expires_at
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        summary.push_str("; expires_at: ");
        summary.push_str(expires_at);
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
