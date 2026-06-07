use core::marker::PhantomData;

/// The state half of a decomposed [`crate::State`].
///
/// Its fields are private so only this crate can create a valid token.
pub struct DecomposedState<S> {
    pub(crate) uid: u64,
    pub(crate) state: PhantomData<fn() -> S>,
    #[cfg(feature = "tracing")]
    pub(crate) trace: Vec<crate::TraceEntry>,
}

/// The data half of a decomposed [`crate::State`].
///
/// Its fields are private so the UID cannot be replaced by callers.
pub struct DecomposedData<T> {
    pub(crate) uid: u64,
    pub(crate) value: T,
}

/// Returned when decomposed state and data tokens do not belong together.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecomposeError;
