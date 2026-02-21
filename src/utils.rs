pub trait Sealed {}

impl<T> Sealed for &mut T where T: Sealed {}

/// A helper trait to perform compile-time checks on bit-packed fields.
pub trait BitPackedCheck<const BITS: u8> {
    /// Performs the check.
    const CHECK: ();
}

impl<const BITS: u8, T> BitPackedCheck<BITS> for T {
    const CHECK: () = assert!(
        BITS > 0 && BITS <= 64 && BITS as usize <= core::mem::size_of::<Self>() * 8,
        "Bit width must be 1-64 and not exceed type size"
    );
}
