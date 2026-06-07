/// State policy permitting [`crate::State`] to clone its runtime value.
///
/// States implement this automatically. A definition crate can opt out with:
///
/// ```ignore
/// impl !StateClone for Sensitive {}
/// ```
pub auto trait StateClone {}

/// State policy permitting [`crate::State`] to copy its runtime value.
///
/// States implement this automatically. A definition crate can opt out with:
///
/// ```ignore
/// impl !StateCopy for Connected {}
/// ```
pub auto trait StateCopy {}
