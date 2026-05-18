use serde::{Deserialize, Serialize};

pub const WITNESS_RECEIPT_SCHEMA_VERSION: &str = "0.1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessReceipt {
    pub schema_version: String,
    pub card_id: String,
    pub tool: String,
    pub strength: String,
    pub author: Option<String>,
    pub recorded_at: Option<String>,
    pub expires_at: Option<String>,
    pub summary: Option<String>,
    pub command: Option<String>,
    pub limitations: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MiriReceiptInput {
    pub card_id: String,
    pub output: String,
    pub author: String,
    pub recorded_at: String,
    pub expires_at: String,
    pub command: String,
    pub limitations: Vec<String>,
}

impl WitnessReceipt {
    pub fn validate(&self) -> Result<(), String> {
        validate_required(&self.schema_version, "schema_version")?;
        validate_required(&self.card_id, "card_id")?;
        validate_required(&self.tool, "tool")?;
        validate_tool(&self.tool)?;
        validate_strength(&self.strength)?;
        if !looks_like_counted_card_id(&self.card_id) {
            return Err("card_id must be an exact counted UR-* identity ending in -cN".to_string());
        }
        let author = validate_required_option(&self.author, "author")?;
        validate_required(author, "author")?;
        let recorded_at = validate_required_option(&self.recorded_at, "recorded_at")?;
        let expires_at = validate_required_option(&self.expires_at, "expires_at")?;
        validate_utc_timestamp(recorded_at, "recorded_at")?;
        validate_date(expires_at, "expires_at")?;
        if expires_at < &recorded_at[..10] {
            return Err("`expires_at` must be on or after the `recorded_at` date".to_string());
        }
        Ok(())
    }

    pub fn evidence_summary(&self) -> String {
        let mut summary = format!(
            "Imported {} receipt with `{}` strength",
            self.tool, self.strength
        );
        if let Some(detail) = self.summary.as_deref().filter(|value| !value.is_empty()) {
            summary.push_str(": ");
            summary.push_str(detail);
        }
        if let Some(author) = self.author.as_deref().filter(|value| !value.is_empty()) {
            summary.push_str("; author: ");
            summary.push_str(author);
        }
        if let Some(recorded_at) = self
            .recorded_at
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            summary.push_str("; recorded_at: ");
            summary.push_str(recorded_at);
        }
        if let Some(expires_at) = self.expires_at.as_deref().filter(|value| !value.is_empty()) {
            summary.push_str("; expires_at: ");
            summary.push_str(expires_at);
        }
        if let Some(command) = self.command.as_deref().filter(|value| !value.is_empty()) {
            summary.push_str("; command: ");
            summary.push_str(command);
        }
        if let Some(limitations) = &self.limitations
            && !limitations.is_empty()
        {
            summary.push_str("; limitations: ");
            summary.push_str(&limitations.join("; "));
        }
        summary
    }

    pub fn to_pretty_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map(|mut text| {
                text.push('\n');
                text
            })
            .map_err(|err| format!("serialize witness receipt failed: {err}"))
    }

    pub fn from_miri_output(input: MiriReceiptInput) -> Result<Self, String> {
        validate_miri_success_output(&input.output)?;
        validate_required(&input.command, "command")?;
        if !input.command.to_ascii_lowercase().contains("miri") {
            return Err("Miri receipt command must mention `miri`".to_string());
        }
        let mut limitations = vec![
            "saved-output adapter; unsafe-review did not run Miri".to_string(),
            "receipt strength is `ran`; site reach is not claimed".to_string(),
        ];
        limitations.extend(input.limitations);
        let receipt = Self {
            schema_version: WITNESS_RECEIPT_SCHEMA_VERSION.to_string(),
            card_id: input.card_id,
            tool: "miri".to_string(),
            strength: "ran".to_string(),
            author: Some(input.author),
            recorded_at: Some(input.recorded_at),
            expires_at: Some(input.expires_at),
            summary: Some("saved Miri output reported `test result: ok`".to_string()),
            command: Some(input.command),
            limitations: Some(limitations),
        };
        receipt.validate()?;
        Ok(receipt)
    }
}

fn is_supported_receipt_strength(value: &str) -> bool {
    matches!(
        value,
        "configured" | "ran" | "test_targeted" | "site_reached"
    )
}

fn is_supported_receipt_tool(value: &str) -> bool {
    matches!(
        value,
        "miri"
            | "cargo-careful"
            | "asan"
            | "msan"
            | "tsan"
            | "lsan"
            | "loom"
            | "shuttle"
            | "kani"
            | "crux"
            | "human-deep-review"
            | "unsupported"
    )
}

fn validate_required(value: &str, key: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("`{key}` is required"))
    } else {
        Ok(())
    }
}

fn validate_required_option<'a>(value: &'a Option<String>, key: &str) -> Result<&'a str, String> {
    let Some(value) = value.as_deref() else {
        return Err(format!("`{key}` is required"));
    };
    validate_required(value, key)?;
    Ok(value)
}

fn validate_strength(value: &str) -> Result<(), String> {
    if is_supported_receipt_strength(value) {
        Ok(())
    } else {
        Err(format!("uses unknown receipt strength `{value}`"))
    }
}

fn validate_tool(value: &str) -> Result<(), String> {
    if is_supported_receipt_tool(value) {
        Ok(())
    } else {
        Err(format!("uses unknown receipt tool `{value}`"))
    }
}

fn validate_miri_success_output(output: &str) -> Result<(), String> {
    if output.trim().is_empty() {
        return Err("saved Miri output is empty".to_string());
    }
    let lower = output.to_ascii_lowercase();
    for needle in [
        "undefined behavior",
        "test result: failed",
        "failures:",
        "panicked at",
        "error:",
    ] {
        if lower.contains(needle) {
            return Err(format!(
                "saved Miri output contains failure marker `{needle}`"
            ));
        }
    }
    if !lower.contains("test result: ok") {
        return Err("saved Miri output must contain `test result: ok`".to_string());
    }
    Ok(())
}

fn validate_utc_timestamp(value: &str, key: &str) -> Result<(), String> {
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
            "`{key}` must use UTC timestamp format YYYY-MM-DDTHH:MM:SSZ"
        ));
    }
    validate_date(&value[..10], key)?;
    validate_range(decimal_at(value, 11, 2), 0, 23, key)?;
    validate_range(decimal_at(value, 14, 2), 0, 59, key)?;
    validate_range(decimal_at(value, 17, 2), 0, 59, key)
}

fn validate_date(value: &str, key: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    let valid_shape = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && [0, 1, 2, 3, 5, 6, 8, 9]
            .iter()
            .all(|index| bytes[*index].is_ascii_digit());
    if !valid_shape {
        return Err(format!("`{key}` must use date format YYYY-MM-DD"));
    }
    validate_range(decimal_at(value, 0, 4), 1, 9999, key)?;
    validate_range(decimal_at(value, 5, 2), 1, 12, key)?;
    validate_range(decimal_at(value, 8, 2), 1, 31, key)
}

fn decimal_at(value: &str, start: usize, len: usize) -> Option<u32> {
    value.get(start..start + len)?.parse().ok()
}

fn validate_range(value: Option<u32>, min: u32, max: u32, key: &str) -> Result<(), String> {
    let Some(value) = value else {
        return Err(format!("`{key}` contains an invalid number"));
    };
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("`{key}` is out of range"))
    }
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

    #[test]
    fn witness_receipt_json_round_trips_and_validates() -> Result<(), String> {
        let receipt = fixture_receipt();
        receipt.validate()?;

        let json = receipt.to_pretty_json()?;
        let decoded: WitnessReceipt = serde_json::from_str(&json)
            .map_err(|err| format!("deserialize receipt failed: {err}"))?;

        assert_eq!(decoded, receipt);
        assert!(decoded.evidence_summary().contains("Imported miri receipt"));
        assert!(decoded.evidence_summary().contains("fixture only"));
        Ok(())
    }

    #[test]
    fn witness_receipt_validation_rejects_unknown_tool() {
        let mut receipt = fixture_receipt();
        receipt.tool = "proof-bot".to_string();

        assert!(
            receipt
                .validate()
                .err()
                .unwrap_or_default()
                .contains("unknown receipt tool")
        );
    }

    #[test]
    fn witness_receipt_validation_rejects_missing_author() {
        let mut receipt = fixture_receipt();
        receipt.author = None;

        assert!(
            receipt
                .validate()
                .err()
                .unwrap_or_default()
                .contains("`author` is required")
        );
    }

    #[test]
    fn miri_receipt_from_saved_output_uses_ran_strength_without_site_reach() -> Result<(), String> {
        let receipt = WitnessReceipt::from_miri_output(MiriReceiptInput {
            card_id: "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1"
                .to_string(),
            output: "running 1 test\ntest read_header ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; finished in 0.01s\n"
                .to_string(),
            author: "core/fixtures".to_string(),
            recorded_at: "2026-05-18T00:00:00Z".to_string(),
            expires_at: "2026-08-18".to_string(),
            command: "cargo +nightly miri test read_header".to_string(),
            limitations: vec!["fixture only".to_string()],
        })?;

        assert_eq!(receipt.tool, "miri");
        assert_eq!(receipt.strength, "ran");
        assert_eq!(
            receipt.summary.as_deref(),
            Some("saved Miri output reported `test result: ok`")
        );
        let limitations = receipt.limitations.as_ref().ok_or("missing limitations")?;
        assert!(
            limitations
                .iter()
                .any(|item| item.contains("unsafe-review did not run Miri"))
        );
        assert!(
            limitations
                .iter()
                .any(|item| item.contains("site reach is not claimed"))
        );
        assert!(limitations.iter().any(|item| item == "fixture only"));
        Ok(())
    }

    #[test]
    fn miri_receipt_from_saved_output_rejects_failure_markers() {
        let result = WitnessReceipt::from_miri_output(MiriReceiptInput {
            card_id: "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1"
                .to_string(),
            output: "error: Undefined Behavior: pointer must be aligned\n".to_string(),
            author: "core/fixtures".to_string(),
            recorded_at: "2026-05-18T00:00:00Z".to_string(),
            expires_at: "2026-08-18".to_string(),
            command: "cargo +nightly miri test read_header".to_string(),
            limitations: Vec::new(),
        });

        assert!(result.err().unwrap_or_default().contains("failure marker"));
    }

    #[test]
    fn miri_receipt_from_saved_output_requires_miri_command() {
        let result = WitnessReceipt::from_miri_output(MiriReceiptInput {
            card_id: "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1"
                .to_string(),
            output: "test result: ok. 1 passed; 0 failed; finished in 0.01s\n".to_string(),
            author: "core/fixtures".to_string(),
            recorded_at: "2026-05-18T00:00:00Z".to_string(),
            expires_at: "2026-08-18".to_string(),
            command: "cargo test read_header".to_string(),
            limitations: Vec::new(),
        });

        assert!(
            result
                .err()
                .unwrap_or_default()
                .contains("must mention `miri`")
        );
    }

    fn fixture_receipt() -> WitnessReceipt {
        WitnessReceipt {
            schema_version: WITNESS_RECEIPT_SCHEMA_VERSION.to_string(),
            card_id: "UR-crate-src-lib-rs-owner-operation-raw_pointer_read-read-deadbeef1234-alignment-c1"
                .to_string(),
            tool: "miri".to_string(),
            strength: "ran".to_string(),
            author: Some("core/fixtures".to_string()),
            recorded_at: Some("2026-05-18T00:00:00Z".to_string()),
            expires_at: Some("2026-08-18".to_string()),
            summary: Some("focused witness passed".to_string()),
            command: Some("cargo +nightly miri test read_header".to_string()),
            limitations: Some(vec!["fixture only".to_string()]),
        }
    }
}
