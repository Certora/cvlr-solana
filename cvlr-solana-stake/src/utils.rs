//! Utilities
use cvlr_asserts::cvlr_assume;

/// Adapter to Prover-friendly Write trait
pub struct CvlrStdIoWrite<'a>(pub &'a mut [u8]);

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