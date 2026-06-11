# MagicStateMachines

MagicStateMachines provides typestate wrappers for compiler-enforced state
machines whose contract can live in a separate crate from the runtime
implementation.

The library is nightly-only because it uses `arbitrary_self_types`.

This library in the default configuration does not perform any `unsafe`. If you want to use thin-pointers for dynamic dispatch of union state transitions, the feature dynZST can be used which relies on unsafe, but is very small and can be easily audited.

All abstractions in this library are zero-cost. The state machines exist only at compile-time and are shown to be well optimizable by the compiler. Only when using state-machines across boundaries that the compiler can no longer prove: e.g. behind a smart-pointer like Arc or Rc the state incurs storage cost and requires dynamic dispatch. However the dynamic dispatch only happens at the boundary at which the state-space gets restricted. Without any restrictions or after a state is proven concrete, all nominally dynamic dispatches are via the compile-time type system back to static dispatch. With other words, calling a dynamically dispatched transition with a concrete state will always be a zero-cost static dispatch, that the compiler can simplify or choose to inline.

The main achievement of this crate is, that it allows using state-machines nearly without any boilerplate, and no penalty to coding ergonomics. A type that so far did not have a state-machine can be retrofitted without changing the overall implementation if it already adhered to the implied state-machine. Async, generics, &mut (SMut), & (&SRef), pin<&mut> (SPinMut), moving (SMove), smart-pointer, and runtime borrow-checking primitives, even traits are all supported. (default fn on traits are NOT supported)

As type-system enforced state-machines change the type, every function that transitions state must always return self. This requires that &mut like borrowing be facilitated behind a guard which must specialise on the backing storage e.g. Mutex, RwLock or RefCell. For rust std types implementations are provided, for third party they can be implemented without restrictions even for foreign types, as wrappers are mostly defined by a ZST which gets around the foreign type restriction similarly to newtypes.

I generally recommend that state-machines are defined in their own crate, for sake of separation of concerns as well as speeding up compile time (which is fast either way). State machine definitions function analogous to traits, in that they define a contract that implementing types must fulfil. They are however private contracts as they do not provide an interface that could be used by consumers of an implementation. Consumers very much can benefit from the compile-time enforcement though.

In general, this crate can be relied upon for safety proofs. Be aware however as rust does not have linear types, any guard can always be mem::forget()ten.

If you have forbidden unsafe, please be aware that the library exposes some unsafe functions, these themselves do not perform anything unsafe, but are marked as unsafe as they are escape hatches to get around the compiler-enforced transition and initial state rules. The same could be achieved by an API consumer calling the unsafe transmute function, the difference is that the library's unsafe functions are completely implemented in safe rust, allowing API consumers to unsafely force a state without having to prove that calling transmute would be safe (which it may not, we do not guarantee this). If you find that there is no unsafe function exposed by the API for your use case, then be aware that most likely what you are trying to achieve in fact is unsafe and would be undefined behaviour. Calling any of the library's unsafe functions only can actually constitute undefined behaviour if a state-machine is used to prove that some real unsafe functions are safe to call. In that case unsafely forcing a state invalidates that proof, and makes the real unsafe function call possibly be undefined behavior. Some of the macros generate convinience functions that are marked unsafe, but marcos never call unsafe functions. You can disable this with the gen_no_unsafe feature.

## Contract Crate

Define the stand-in type, state markers, initial states, allowed transitions,
and state unions:

```rust
use magicstatemachines::{StateMachineDefinition, States};

pub struct ConnectionStandin;

pub mod states {
    use magicstatemachines::States;

    States! {
        Disconnected;
        Connected;
        Authenticated;
    }
}

use states::{Authenticated, Connected, Disconnected};

StateMachineDefinition! {
    for ConnectionStandin;

    pub Initial: Disconnected;

    transition Disconnected => Connected();
    transition Connected => Authenticated(user: String);
    transition Connected => Disconnected();
    transition Authenticated => Connected | Disconnected();

    union Online: Connected | Authenticated;
}
```

The contract crate owns the stand-in and states, so downstream crates cannot
add extra transitions.

## Implementation Crate

Connect a runtime type to the contract and implement methods with state-typed
receivers:

```rust
use magicstatemachines::{SMut, State, StateMachineImpl, transition};
use contract::{
    ConnectionStandin, InOnline, Online,
    states::{Authenticated, Connected, Disconnected},
};

pub struct Connection {
    user: Option<String>,
}

StateMachineImpl! {
    Connection: ConnectionStandin;

    transition Disconnected => Connected();

    transition Connected => Authenticated(user: String) {
        self.user = Some(user);
    }

    transition Connected | Authenticated => Disconnected() {
        self.user = None;
    }
}

impl Connection {
    pub fn connect<S>(self: State<S, Self, Disconnected>) -> State<S, Self, Connected>
    where
        S: SMut,
    {
        transition!(self)
    }

    pub fn authenticate<S>(
        self: State<S, Self, Connected>,
        user: impl Into<String>,
    ) -> State<S, Self, Authenticated>
    where
        S: SMut,
    {
        transition!(self, user.into())
    }

    pub fn disconnect<S>(self: State<S, Self, impl InOnline>) -> State<S, Self, Disconnected>
    where
        S: SMut,
    {
        transition!(const Online self)
    }
}
```

`transition!` is only usable in the module where `StateMachineImpl!` generated
the private transition token. Public callers can only transition through the
methods the implementation exposes.

You might have noticed the `InOnline` trait, that is not explicitly defined in this example. It is generated by the `StateMachineDefinition!` macro when the union was defined `union Online: Connected | Authenticated;`

## State Storage

`State<Storage, T, S>` separates the runtime type `T`, current state marker
`S`, and storage backend `Storage`.

Common storage aliases include:

- `SOwned`: directly owned runtime value
- `SBox<T, S>`: boxed owned runtime value
- `SPinBox<T, S>`: pinned boxed runtime value
- `SRcRefCell<T>`: shared `Rc<RefCell<_>>` state
- `SArcMutex<T>`: shared `Arc<Mutex<_>>` state
- `SArcRwLock<T>`: shared `Arc<RwLock<_>>` state

Implementation methods usually constrain storage by capability:

- `SRef`: read-only access to the runtime value
- `SMut`: mutable access and ordinary transitions
- `SPinRef`: pinned shared access
- `SPinMut`: pinned mutable access and pinned transitions
- `SMove`: storage can be moved by value

This lets one method work across owned values, boxes, shared guards, and custom
storage backends.

## State Unions

`StateUnion!` and `StateMachineDefinition! { union ... }` generate:

- a public union marker, for example `Online`
- a sealed membership trait, for example `InOnline`
- a generated enum, for example `OnlineEnum<Storage, T>`

Use the generated `In...` trait in method signatures:

```rust
fn endpoint<S>(self: &State<S, Self, impl InOnline>) -> &str
where
    S: magicstatemachines::SRef,
{
    &self.endpoint
}
```

Use `EnumExt::into_enum` when runtime branching is needed (this incurs the runtime cost of stack allocating the enum):

```rust
use magicstatemachines::EnumExt;

match Online.into_enum(state) {
    OnlineEnum::Connected(connected) => {
        // connected: State<_, Connection, Connected>
    }
    OnlineEnum::Authenticated(authenticated) => {
        // authenticated: State<_, Connection, Authenticated>
    }
}
```

Use `In::into_discriminated` when the return type should remain a
`DiscriminatedState<_, _, Online>`.

## Pinned Transitions

Pinned effects receive `Pin<&mut T>` instead of `&mut T`:

```rust
StateMachineImpl! {
    Connection: ConnectionStandin;

    pinned transition Disconnected => Connected() {
        self.as_mut().mark_connected();
    }
}
```

Call pinned transitions with:

```rust
transition!(pin self)
```

Pinned union transitions are also supported:

```rust
transition!(pin const Online self);
transition!(pin dyn Online self);
```

`pin const` requires every union member to share the same pinned body for the
target transition. `pin dyn` discriminates the current concrete state and runs
that state's pinned body.

## Features

- `dynZST`: stores dyn Trait over zero-sized types as a thin-pointer via my `dynzst` crate  (uses unsafe and relies to )
- `tracing`: records transition source, target, and callsite - no longer zero-cost!
- `unique-rc-arc`: enables `UniqueRc` and `UniqueArc` owned storage backends
- `gen_no_unsafe`: makes macros not generate any unsafe convinience functions (no unsafe is ever called either way, this feature is in case this is a problem to auditors)
- the decompose feature is likely not so great, it allows for a state to be temporarily disconnected from the data, which is runtime enforced by equality on a random sentinel. This likely does break guarantees by this api, because an attacker could brute force a collision. I should consider instead using an atomic counter that errors on overflow. but for now it is what it is.
  - `decompose`: enables `StateOwned::decompose` and `StateOwned::recompose`
  - `decompose-rand`: enables the stable `rand` backend for decomposition IDs
  - `nightly-random`: uses nightly `std::random` for decomposition IDs
## Layout

`StateOwned<T, S>` is transparent over `T` outside of tracing. State markers are
zero-sized, and storage wrappers are designed to preserve the backend layout
where the state can be inferred from the type or from existing runtime storage.
