/// Calls one of the generated transition helpers.
///
/// Supported forms:
///
/// ```ignore
/// transition!(state);
/// transition!(state,);
/// transition!(state, user.into());
/// transition!(state, user.into(),);
/// transition!(const Online state);
/// transition!(const Online state,);
/// transition!(dyn Online state);
/// transition!(dyn some::Marker, state);
/// ```
#[macro_export]
macro_rules! transition {
    (const $marker:ident $state:expr) => {
        $crate::transition!(@call $state._magicsm_transitionConst($marker))
    };
    (const $marker:ident $state:expr,) => {
        $crate::transition!(@call $state._magicsm_transitionConst($marker))
    };
    (const $marker:ident $state:expr, $($arg:expr),+ $(,)?) => {
        $crate::transition!(@call $state._magicsm_transitionConst($marker), $($arg),+)
    };
    (dyn $marker:ident $state:expr) => {
        $crate::transition!(@call $state._magicsm_transitionDyn($marker))
    };
    (dyn $marker:ident $state:expr,) => {
        $crate::transition!(@call $state._magicsm_transitionDyn($marker))
    };
    (dyn $marker:ident $state:expr, $($arg:expr),+ $(,)?) => {
        $crate::transition!(@call $state._magicsm_transitionDyn($marker), $($arg),+)
    };
    (const $marker:path, $state:expr) => {
        $crate::transition!(@call $state._magicsm_transitionConst($marker))
    };
    (const $marker:path, $state:expr,) => {
        $crate::transition!(@call $state._magicsm_transitionConst($marker))
    };
    (const $marker:path, $state:expr, $($arg:expr),+ $(,)?) => {
        $crate::transition!(@call $state._magicsm_transitionConst($marker), $($arg),+)
    };
    (dyn $marker:path, $state:expr) => {
        $crate::transition!(@call $state._magicsm_transitionDyn($marker))
    };
    (dyn $marker:path, $state:expr,) => {
        $crate::transition!(@call $state._magicsm_transitionDyn($marker))
    };
    (dyn $marker:path, $state:expr, $($arg:expr),+ $(,)?) => {
        $crate::transition!(@call $state._magicsm_transitionDyn($marker), $($arg),+)
    };
    ($state:expr) => {
        $crate::transition!(@call $state._magicsm_transition())
    };
    ($state:expr,) => {
        $crate::transition!(@call $state._magicsm_transition())
    };
    ($state:expr, $($arg:expr),+ $(,)?) => {
        $crate::transition!(@call $state._magicsm_transition(), $($arg),+)
    };
    (@call $call:expr) => {
        $call.call(())
    };
    (@call $call:expr, $($arg:expr),+ $(,)?) => {
        $call.call(($($arg,)+))
    };
}
