// Policy parsing is deliberately small in v0.1. The public policy schemas live in
// `policy/*.toml`; the analyzer currently treats them as advisory documentation.
// Baseline/suppression matching will be wired in a later spec-backed slice.
#[derive(Clone, Debug, Default)]
#[allow(
    dead_code,
    reason = "Reserved for spec-backed policy parsing once baseline and suppression matching are implemented."
)]
pub(crate) struct PolicyState;
