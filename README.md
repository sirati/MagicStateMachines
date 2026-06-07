# statemachines

`statemachines` provides a transparent typestate wrapper for compiler-enforced
state machines. The state-machine contract and its runtime implementation live
in separate crates.

The workspace contains:

- `statemachines`: the reusable `State<Storage, T, S>` wrapper
- `test_def`: only the stand-in ZST, states, allowed transitions, and unions
- `test_use`: the runtime type and all method implementations

The Nix flake provides the latest nightly Rust toolchain from `rust-overlay`,
including Cargo, Clippy, rustfmt, rust-src, rust-analyzer, and cargo-nextest.
No rustup toolchain is used.

Enter the development environment before running project commands:

```console
nix develop
cargo test --workspace
cargo run -p test_use
```

The definition crate fixes the transition graph:

```rust
impl Transition<Disconnected, Connected> for ConnectionStandin {}
```

Transitions can declare required arguments with an associated function type:

```rust
impl Transition<Connected, Authenticated> for ConnectionStandin {
    type F = fn(String);
}

let authenticated =
    connected.authenticate("alice");
```

The associated type defaults to `fn()`. The implementation macro generates the
private transition capability and module-local `transition()` helpers:

```rust
statemachines::StateMachineImpl!(Connection: ConnectionStandin);
```

Those helpers return a one-shot callable whose `FnOnce` implementation uses the
`rust-call` ABI. The underlying public `statemachines::transition` and
`transition_mut` functions require the generated token, whose constructor
remains private to the implementation module. Safe caller code therefore
cannot bypass the implementation's transition methods.

Because `Transition`, `ConnectionStandin`, and the states are owned by other
crates, the implementation crate cannot add another transition.
The definition separately declares the permitted initial state:

```rust
impl Initial<Disconnected> for ConnectionStandin {}
```

A state can be split while preserving its type-level identity:

```rust
let (state, data) = connection.decompose();
let connection = StateOwned::recompose(state, data)?;
```

Both opaque values carry the same random `u64`. Recomposition rejects tokens
from different decompositions.

The project uses nightly Rust because arbitrary self types are unstable:

```rust
pub fn connect(
    self: State<StorageStateOwned, Self, states::Disconnected>,
) -> State<StorageStateOwned, Self, states::Connected> {
    self.transition()()
}
```

Operations shared by several states use a generated union marker:

```rust
StateUnion!(Online: Connected | Authenticated);
```

An explicitly named value-carrying enum can be generated together with the
trait or by itself:

```rust
StateUnion!(Online, enum OnlineEnum: Connected | Authenticated);
StateUnion!(enum OnlineEnum: Connected | Authenticated);
```

Union traits are sealed. They can inherit one or more previously defined union
traits, with `+` separating supertraits and `|` separating member states:

```rust
StateUnion!(All: Disconnected | Connected | Authenticated);
StateUnion!(Online: All, Connected | Authenticated);
StateUnion!(Specific: All + Online, Connected | Authenticated);
```

Each union has a generated joint ZST state. `OnlineEnum<Storage, T>` preserves
the concrete variant while every variant exposes the same
`State<Storage, T, JointOnlineState>` view. Concrete states convert into the
enum with `Into`, and matching variants recover their concrete state with
`into_state()`.

Functions with one success and one failure state can use the shorter result
alias:

```rust
fn try_connect<S>(
    self: State<S, Self, Disconnected>,
) -> SResult<S, Self, Connected, Disconnected>;
```

Generated union enums dereference to their common joint `State`. Existing
inherent methods restricted by the union trait therefore work directly:

```rust
fn endpoint(
    self: &State<impl SRef, Self, impl Online>,
) -> &str {
    &self.endpoint
}

let online: OnlineEnum<_, Connection> = connected.into();
online.endpoint();
```

Consuming `into_joint()` exposes the joint state. It has a transition to a
target only when every member state has that transition with the same
associated function signature. This permits generic shared transitions without
weakening the state-machine contract.

The concrete owned state marker is a zero-sized `PhantomData`;
`#[repr(transparent)]` makes `StateOwned<T, S>` layout-compatible with `T`.

`State<Storage, T, S>` generalizes over storage backends. The standard backends
include directly owned values, `Box<T>`, `Pin<Box<T>>`, `UniqueRc<T>`,
`UniqueArc<T>`, and mutable shared-state guards. That lets one implementation
method preserve the storage backend:

```rust
fn connect<Storage>(
    self: State<Storage, Self, Disconnected>,
) -> State<Storage, Self, Connected>
where
    Storage: SRef,
{
    self.transition()()
}
```

Shared receivers such as `Rc<T>`, `Arc<T>`, and `&T` are deliberately not
supported as state storage: they could alias one runtime value with
independently evolving state tokens. `StateOwned<T, S>` implements `Clone` or
`Copy` when `T` does and the state permits it through the default `StateClone`
and `StateCopy` auto traits. A definition can opt a state out:

```rust
impl !StateCopy for Connected {}
impl !StateClone for Authenticated {}
```

Opting out of `StateClone` also prevents `Copy`, as required by Rust's `Copy`
contract. `core::ptr::Unique<T>` is not supported because it is a `Copy`
raw-pointer primitive rather than an arbitrary-self receiver.

Shared and interior-mutable values use `SharedState<Storage, T>`. The standard
aliases make the backend explicit: `RefCellState<T>` uses `Rc<RefCell<_>>`,
while `MutexState<T>` uses `Arc<Mutex<_>>`. These keep one authoritative erased
state marker next to
the data. Typed guards verify the current state, and a mutable guard commits its
final state back to the parent when dropped:

```rust
let shared = RefCellState::new::<Disconnected>(connection);
let guard = shared.borrow_mut::<Disconnected>()?;
let connected = guard.connect();
drop(connected);

let connected = shared.borrow::<Connected>()?;
```

Erased state markers are stored with the pinned `dynzst` dependency.
Alternative lock or cell families implement the non-generic `SharedStorage`
trait using its `Storage<T>` GAT. Ownership and synchronization compose as
`RcState<MyStorage, T>` or `ArcState<MyStorage, T>`; the storage marker itself
does not contain `T`.

Enable transition tracing with:

```console
cargo run -p test_use --features statemachines/tracing
```

With tracing enabled, each `StateOwned` stores a `Vec<TraceEntry>`. Every entry owns
the source and destination as `&'static dyn statemachines::tracing::State` and
also stores the caller location. The sealed tracing trait is implemented only
for ZSTs; references contain only the marker's trait-object metadata. The trace is
preserved across decomposition and recomposition and is available through
`StateOwned::trace()`. Tracing adds runtime storage, and traced states cannot
implement `Copy`.
