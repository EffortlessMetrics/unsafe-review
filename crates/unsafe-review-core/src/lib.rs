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

pub use api::{
    AnalysisMode, AnalyzeInput, AnalyzeOutput, DiffSource, PolicyMode, Scope, analyze,
    collect_context, explain_card, render_human, render_json, render_markdown, render_pr_summary,
};
pub use domain::{
    CardId, Confidence, ContractEvidence, DischargeEvidence, HazardKind, MissingEvidence,
    NextAction, Priority, ReachEvidence, RelatedTest, ReviewCard, ReviewClass, SafetyObligation,
    SourceLocation, UnsafeOperation, UnsafeSite, WitnessEvidence, WitnessKind, WitnessRoute,
};
