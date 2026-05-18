pub(crate) mod classify;
pub(crate) mod evidence;
pub(crate) mod obligations;
pub(crate) mod pipeline;
pub(crate) mod scanner;
#[allow(
    dead_code,
    reason = "Stable syntax substrate is introduced before the scanner cutover uses it."
)]
pub(crate) mod syntax;
pub(crate) mod witness;
