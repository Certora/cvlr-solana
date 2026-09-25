//! Utilities
//!
//! A `std::io::Write` adapter for serializing into account data.
use cvlr_asserts::cvlr_assume;

/// Adapter to Prover-friendly Write trait
///
/// Writes into the wrapped slice from the front, advancing past what it has
/// written, like `std::io::Write for &mut [u8]`. Instead of returning an error
/// (or writing a short count) when the data does not fit, it *assumes* that
/// the data fits, so executions that would overflow the slice are dropped.
/// This keeps error handling out of the serialization code that the Prover
/// analyzes; used to borsh-serialize `StakeStateV2` into account data.
pub struct CvlrStdIoWrite<'a>(
    /// The part of the buffer not yet written.
    pub &'a mut [u8],
);

impl std::io::Write for CvlrStdIoWrite<'_> {
    /// Implementation of `write` that checks for length first
    #[inline]
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        cvlr_assume!(data.len() <= self.0.len());
        let amt = data.len();
        let (a, b) = std::mem::take(&mut self.0).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        self.0 = b;
        Ok(amt)
    }

    // Implementation of `write_all` that assumes write does not fail
    #[inline]
    fn write_all(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.write(data).unwrap();
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
