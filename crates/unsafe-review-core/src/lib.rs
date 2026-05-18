#![forbid(unsafe_code)]
//! Core SDK and analysis engine for `unsafe-review`.
//!
//! The public API is intentionally small: build an [`AnalyzeInput`], call
//! [`analyze`], and render or consume the returned [`AnalyzeOutput`].

mod analysis;
pub mod api;
mod domain;
mod input;
mod output;
mod policy;
mod util;

pub use api::OutcomeReport;
pub use api::{
    AnalysisMode, AnalyzeInput, AnalyzeOutput, DiffSource, PolicyMode, Scope, analyze,
    collect_context, compare_outcome_json, explain_card, render_comment_plan, render_human,
    render_json, render_lsp, render_markdown, render_outcome_json, render_outcome_markdown,
    render_pr_summary, render_sarif, render_witness_plan, validate_witness_receipts,
};
pub use domain::{
    CardId, CargoCarefulReceiptInput, ConcurrencyReceiptInput, Confidence, ContractEvidence,
    DischargeEvidence, HazardKind, MiriReceiptInput, MissingEvidence, NextAction, Priority,
    ProofReceiptInput, ReachEvidence, RelatedTest, ReviewCard, ReviewClass, SafetyObligation,
    SanitizerReceiptInput, SourceLocation, UnsafeOperation, UnsafeSite,
    WITNESS_RECEIPT_SCHEMA_VERSION, WitnessEvidence, WitnessKind, WitnessReceipt, WitnessRoute,
};
