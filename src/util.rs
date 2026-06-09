use crate::{In, State, StateMachineImpl, StateStorage, StateUnionDiscriminant};

/// Convenience extension for converting a state into a union's generated enum.
///
/// This is implemented for every generated union marker. It is most useful
/// when the marker value is already in scope and you want the concrete enum
/// immediately, without spelling the associated [`In`](crate::In) conversion
/// and then calling `discriminate()` yourself:
///
/// ```ignore
/// use magicstatemachines::EnumExt;
/// use test_def::{Online, OnlineEnum};
///
/// let online = Online.into_enum(state);
///
/// match online {
///     OnlineEnum::Connected(connected) => {
///         // `connected` has the concrete `Connected` state marker.
///     }
///     OnlineEnum::Authenticated(authenticated) => {
///         // `authenticated` has the concrete `Authenticated` state marker.
///     }
/// }
/// ```
pub trait EnumExt: StateUnionDiscriminant {
    /// Converts `state` into this union's generated enum.
    ///
    /// The input state may be any concrete state that implements `In<Self>`.
    /// The output is the enum associated with this marker, for example
    /// `OnlineEnum<S, Connection>` for marker `Online`.
    fn into_enum<S, T, Current>(
        self,
        state: State<S, T, Current>,
    ) -> <Self as StateUnionDiscriminant>::Enum<S, T>
    where
        S: StateStorage,
        T: StateMachineImpl,
        Current: In<Self>,
    {
        <Current as In<Self>>::into_discriminated(state).discriminate()
    }
}

impl<T: StateUnionDiscriminant> EnumExt for T {}
