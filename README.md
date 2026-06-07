# statemachines

`statemachines` provides a transparent typestate wrapper for compiler-enforced
state machines. The state-machine contract and its runtime implementation live
in separate crates.

The workspace contains:

- `statemachines`: the reusable `State<T, S>` wrapper
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

let authenticated = connected.transition::<Authenticated>()("alice".into());
```

The associated type defaults to `fn()`. `transition()` returns a one-shot
callable whose `FnOnce` implementation uses the `rust-call` ABI, giving normal
function-call syntax and compiler-enforced parameters.

Because `Transition`, `ConnectionStandin`, and the states are owned by other
crates, the implementation crate cannot add another transition.
The definition separately declares the permitted initial state:

```rust
impl Initial<Disconnected> for ConnectionStandin {}
```

A state can be split while preserving its type-level identity:

```rust
let (state, data) = connection.decompose();
let connection = State::recompose(state, data)?;
```

Both opaque values carry the same random `u64`. Recomposition rejects tokens
from different decompositions.

The project uses nightly Rust because arbitrary self types are unstable:

```rust
pub fn connect(
    self: State<Self, states::Disconnected>,
) -> State<Self, states::Connected> {
    self.transition()()
}
```

Operations shared by several states use a generated union marker:

```rust
StateUnion!(Online: Connected + Authenticated);
```

The state marker is a zero-sized `PhantomData`; `#[repr(transparent)]` makes
`State<T, S>` layout-compatible with `T`.

`State` also preserves arbitrary-self receiver chains for uniquely owned
storage. `Box<T>`, `Pin<Box<T>>`, `&mut T`, `UniqueRc<T>`, and `UniqueArc<T>`
forward the implementation contract, allowing receivers such as:

```rust
fn connect_boxed(
    self: State<Box<Self>, Disconnected>,
) -> State<Box<Self>, Connected> {
    self.transition()()
}
```

Shared receivers such as `Rc<T>`, `Arc<T>`, and `&T` are deliberately not
supported as state storage: they could alias one runtime value with
independently evolving state tokens. `State<T, S>` implements `Clone` or `Copy`
when `T` does and the state permits it through the default `StateClone` and
`StateCopy` auto traits. A definition can opt a state out:

```rust
impl !StateCopy for Connected {}
impl !StateClone for Authenticated {}
```

Opting out of `StateClone` also prevents `Copy`, as required by Rust's `Copy`
contract. `core::ptr::Unique<T>` is not supported because it is a `Copy`
raw-pointer primitive rather than an arbitrary-self receiver.

Enable transition tracing with:

```console
cargo run -p test_use --features statemachines/tracing
```

With tracing enabled, each `State` stores a `Vec<TraceEntry>`. Every entry owns
the source and destination as `&'static dyn statemachines::tracing::State` and
also stores the caller location. The sealed tracing trait is implemented only
for ZSTs; references contain only the marker's trait-object metadata. The trace is
preserved across decomposition and recomposition and is available through
`State::trace()`. Tracing adds runtime storage, and traced states cannot
implement `Copy`.
