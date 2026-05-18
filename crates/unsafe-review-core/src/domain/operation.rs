use super::SourceLocation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnsafeSiteKind {
    UnsafeBlock,
    UnsafeFn,
    UnsafeTrait,
    UnsafeImpl,
    UnsafeImplSend,
    UnsafeImplSync,
    ExternBlock,
    FfiCall,
    StaticMut,
    Operation,
}

impl UnsafeSiteKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsafeBlock => "unsafe_block",
            Self::UnsafeFn => "unsafe_fn",
            Self::UnsafeTrait => "unsafe_trait",
            Self::UnsafeImpl => "unsafe_impl",
            Self::UnsafeImplSend => "unsafe_impl_send",
            Self::UnsafeImplSync => "unsafe_impl_sync",
            Self::ExternBlock => "extern_block",
            Self::FfiCall => "ffi_call",
            Self::StaticMut => "static_mut",
            Self::Operation => "operation",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationFamily {
    RawPointerDeref,
    RawPointerRead,
    RawPointerReadUnaligned,
    RawPointerWrite,
    PointerArithmetic,
    CopyNonOverlapping,
    SliceFromRawParts,
    StrFromUtf8Unchecked,
    MaybeUninitAssumeInit,
    VecSetLen,
    Transmute,
    Zeroed,
    BoxFromRaw,
    NonNullUnchecked,
    PinUnchecked,
    GetUnchecked,
    UnsafeImplSendSync,
    Ffi,
    StaticMut,
    InlineAsm,
    TargetFeature,
    Unknown,
}

impl OperationFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RawPointerDeref => "raw_pointer_deref",
            Self::RawPointerRead => "raw_pointer_read",
            Self::RawPointerReadUnaligned => "raw_pointer_read_unaligned",
            Self::RawPointerWrite => "raw_pointer_write",
            Self::PointerArithmetic => "pointer_arithmetic",
            Self::CopyNonOverlapping => "copy_nonoverlapping",
            Self::SliceFromRawParts => "slice_from_raw_parts",
            Self::StrFromUtf8Unchecked => "str_from_utf8_unchecked",
            Self::MaybeUninitAssumeInit => "maybe_uninit_assume_init",
            Self::VecSetLen => "vec_set_len",
            Self::Transmute => "transmute",
            Self::Zeroed => "zeroed",
            Self::BoxFromRaw => "box_from_raw",
            Self::NonNullUnchecked => "nonnull_unchecked",
            Self::PinUnchecked => "pin_unchecked",
            Self::GetUnchecked => "get_unchecked",
            Self::UnsafeImplSendSync => "unsafe_impl_send_sync",
            Self::Ffi => "ffi",
            Self::StaticMut => "static_mut",
            Self::InlineAsm => "inline_asm",
            Self::TargetFeature => "target_feature",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsafeOperation {
    pub family: OperationFamily,
    pub expression: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsafeSite {
    pub location: SourceLocation,
    pub kind: UnsafeSiteKind,
    pub owner: Option<String>,
    pub visibility: String,
    pub public_api_surface: bool,
    pub changed: bool,
    pub snippet: String,
}
