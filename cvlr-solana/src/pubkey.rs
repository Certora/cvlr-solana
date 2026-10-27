//! Wrappers that make a [`Pubkey`] loggable with `clog!`.

use cvlr_log::cvlr_log_with;
use solana_pubkey::Pubkey;
use std::ops::Deref;

/// A lightweight wrapper around a borrowed [`Pubkey`] that provides
/// convenience methods and logging capabilities.
///
/// `Pubkey` does not implement `CvlrLog`; wrap it to log it:
/// `clog!(Pk(acc.key) => "vault key")`. The key is shown as a 256-bit integer
/// (its 32 bytes read as a little-endian number), so two logged keys can be
/// compared at a glance. `Pk` dereferences to `Pubkey` and converts from and
/// to `&Pubkey`.
pub struct Pk<'a>(
    /// The wrapped key.
    pub &'a Pubkey,
);
impl Pk<'_> {
    /// Converts this borrowed `Pk` into an owned [`PkO`] instance.
    pub fn to_owned(&self) -> PkO {
        self.into()
    }
}

impl<'a> AsRef<Pubkey> for Pk<'a> {
    fn as_ref(&self) -> &'a Pubkey {
        self.0
    }
}

impl cvlr_log::CvlrLog for Pk<'_> {
    /// Logs the wrapped [`Pubkey`] using a provided tag and logger.
    #[inline(always)]
    fn log(&self, tag: &str, logger: &mut cvlr_log::CvlrLogger) {
        let val = cvlr_mathint::NativeInt::from(self.0.as_ref());
        cvlr_log_with(tag, &val, logger);
    }
}

impl Deref for Pk<'_> {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> From<&'a Pubkey> for Pk<'a> {
    fn from(value: &'a Pubkey) -> Self {
        Self(value)
    }
}

impl<'a> From<Pk<'a>> for &'a Pubkey {
    fn from(value: Pk<'a>) -> Self {
        value.0
    }
}

/// An owned wrapper around [`Pubkey`] that complements [`Pk`].
///
/// It dereferences to `Pubkey` but does not implement `CvlrLog`; to log it,
/// log `Pk(&pko)`.
pub struct PkO(Pubkey);
impl PkO {
    /// Creates a new owned wrapper from a [`Pubkey`].
    pub fn new(pk: Pubkey) -> Self {
        Self(pk)
    }
}

impl AsRef<Pubkey> for PkO {
    fn as_ref(&self) -> &Pubkey {
        &self.0
    }
}

impl From<Pubkey> for PkO {
    fn from(value: Pubkey) -> Self {
        Self::new(value)
    }
}

impl From<&Pubkey> for PkO {
    fn from(value: &Pubkey) -> Self {
        Self::new(*value)
    }
}

impl From<&Pk<'_>> for PkO {
    fn from(value: &Pk<'_>) -> Self {
        Self::new(*value.0)
    }
}

impl Deref for PkO {
    type Target = Pubkey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
