# Reference — Verified patterns from official documentation

This file is built incrementally during refactoring. Every entry MUST include:
1. The rule/pattern
2. A code example
3. The source URL from official documentation

**Do not add anything here that isn't verified from an official source.**

---

## Rust Idioms

### Inline format args
Use captured identifiers directly in format strings instead of passing them as separate arguments. More concise and readable.

```rust
// Before (avoid)
let msg = format!("Error: {}", e);

// After (idiomatic)
let msg = format!("Error: {e}");
```

Source: https://doc.rust-lang.org/std/fmt/index.html (captured identifiers section)

### `if let` vs `match`
Use `if let` when you only care about one specific pattern and want to ignore all other cases. Eliminates boilerplate `_ => ()` arms. Use `match` when you need exhaustive checking or handle multiple variants.

```rust
// Before (avoid) - verbose match for a single pattern
match config_max {
    Some(max) => println!("Max is {max}"),
    _ => (),
}

// After (idiomatic)
if let Some(max) = config_max {
    println!("Max is {max}");
}
```

Source: https://doc.rust-lang.org/book/ch06-03-if-let.html

### Iterator `.collect()` for HashMap
Build HashMaps from iterators by collecting tuples `(K, V)`. Use `.zip()` to combine key/value iterators, then `.collect()`.

```rust
use std::collections::HashMap;

// Before (avoid) - manual loop with insert
let mut scores = HashMap::new();
for i in 0..keys.len() {
    scores.insert(keys[i], values[i]);
}

// After (idiomatic)
let scores: HashMap<_, _> = keys.into_iter().zip(values).collect();
```

Source: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect

### `&str` vs `String` in function params (C-CALLER-CONTROL)
If a function does not require ownership, take `&str` instead of `String` so callers don't need to give up ownership or allocate unnecessarily.

```rust
// Before (avoid) - takes ownership unnecessarily
fn greet(name: String) {
    println!("Hello, {name}!");
}

// After (idiomatic) - borrows, caller keeps control
fn greet(name: &str) {
    println!("Hello, {name}!");
}
```

Source: https://rust-lang.github.io/api-guidelines/flexibility.html (C-CALLER-CONTROL)

### Iterator chains vs manual loops
Use iterator adapters (`.iter()`, `.map()`, `.filter()`, `.collect()`) instead of manual `for` loops with `push`. More declarative and composable.

```rust
// Before (avoid)
let mut v2 = Vec::new();
for x in &v1 {
    v2.push(x + 1);
}

// After (idiomatic)
let v2: Vec<_> = v1.iter().map(|x| x + 1).collect();
```

Source: https://doc.rust-lang.org/book/ch13-02-iterators.html

### `thiserror` for custom error types
Use `thiserror`'s derive macro for `std::error::Error` with minimal boilerplate. `#[error("...")]` for Display, `#[from]` for automatic `From` conversions (enabling `?`), `#[source]` for error chaining.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
}
```

Source: https://docs.rs/thiserror/latest/thiserror/

### `#[must_use]` attribute
Annotate functions and types with `#[must_use]` when discarding the return value is almost certainly a bug. The compiler emits a warning if the value is unused.

```rust
#[must_use = "this returns the new value and does not modify the original"]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

add(1, 2);          // WARNING: unused return value
let _ = add(1, 2);  // OK: intentionally discarded
```

Source: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute

## Rust Module Organization

### File-to-module mapping (modern convention)
When splitting `foo.rs` into a module directory, create `foo/mod.rs` (or `foo.rs` at the parent level) and submodule files in `foo/`. The `mod` declaration loads the file — it's not an include.

```rust
// src/lib.rs
mod db;  // loads src/db/mod.rs (or src/db.rs)

// src/db/mod.rs
pub mod connection;  // loads src/db/connection.rs
pub mod schema;      // loads src/db/schema.rs
pub mod query;       // loads src/db/query.rs
```

**Key rules:**
- Cannot have both `db.rs` and `db/mod.rs` — pick one style
- Use `mod` declaration only once per module in the tree
- Directories and files must match the module tree structure

Source: https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html

### Re-exporting with `pub use`
Use `pub use` in `mod.rs` to flatten the public API so callers don't need to know the internal module structure. This lets you refactor internals without breaking external imports.

```rust
// src/db/mod.rs
mod connection;
mod schema;
mod query;

// Re-export everything so `crate::db::ConnectionInfo` still works
pub use connection::*;
pub use schema::*;
pub use query::*;
```

External code can still use `crate::db::ConnectionInfo` instead of `crate::db::connection::ConnectionInfo`.

Source: https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html

### Glob re-exports (`pub use module::*`)
Glob re-exports import all public items from a submodule. Useful in `mod.rs` to maintain backward-compatible API when splitting a file into submodules. Items and named imports shadow glob imports in the same namespace.

```rust
// In mod.rs — re-export all public items from submodules
pub use self::connection::*;
pub use self::schema::*;
```

Source: https://doc.rust-lang.org/reference/items/use-declarations.html

### `mod.rs` vs new-style convention
Two styles exist:
- **Old style**: `src/db/mod.rs` — module body in `mod.rs`
- **New style**: `src/db.rs` alongside `src/db/` directory — module body in `db.rs`

Both are valid; avoid mixing within the same module. The `mod.rs` style is common when the module has many submodules and the root file is mostly declarations and re-exports.

Source: https://doc.rust-lang.org/reference/items/modules.html

## Rust Naming Conventions

### C-CASE: Casing conforms to RFC 430
| Item | Convention |
|------|-----------|
| Modules | `snake_case` |
| Types (structs, enums) | `UpperCamelCase` |
| Traits | `UpperCamelCase` |
| Functions, methods | `snake_case` |
| Constants | `SCREAMING_SNAKE_CASE` |

Acronyms count as one word: `Uuid` not `UUID`. In snake_case, acronyms are lowercased: `is_xid_start`.

```rust
mod connection;       // module: snake_case
struct ConnectionInfo // type: UpperCamelCase
fn get_columns()      // function: snake_case
```

Source: https://rust-lang.github.io/api-guidelines/naming.html#c-case

### C-GETTER: Getter names omit `get_` prefix
Prefer `fn name(&self)` over `fn get_name(&self)`. Reserve `get` only for cases like `Cell::get()` where there is one obvious thing to retrieve.

```rust
// Good
fn connection_info(&self) -> &ConnectionInfo { ... }

// Avoid
fn get_connection_info(&self) -> &ConnectionInfo { ... }
```

Source: https://rust-lang.github.io/api-guidelines/naming.html#c-getter

## Rust `use` Idioms

### Functions: import the parent module
For functions, import the parent module, not the function directly, to make it clear the function isn't locally defined.

```rust
// Idiomatic
use crate::db::connection;
connection::connect(&info);

// Less idiomatic
use crate::db::connection::connect;
connect(&info);
```

Source: https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html

### Structs and enums: import the full path
For structs and enums, import the type directly.

```rust
// Idiomatic
use std::collections::HashMap;
let map = HashMap::new();
```

Source: https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html

### Nested paths to clean up imports
Combine multiple imports from the same module using nested paths.

```rust
// Instead of:
use std::cmp::Ordering;
use std::io;

// Use:
use std::{cmp::Ordering, io};
```

Source: https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html

## Leptos Component Best Practices

### Component definition with `#[component]`
Every component is a function decorated with `#[component]` that takes zero or more arguments (props) and returns `impl IntoView`. The function runs **once** to set up the UI — reactivity comes from signals, not re-running the component.

```rust
#[component]
fn MyComponent(
    /// The label to display.
    #[prop(into)]
    label: Signal<String>,
) -> impl IntoView {
    view! { <span>{label}</span> }
}
```

Source: https://book.leptos.dev/view/01_basic_component.html

### Props: ReadSignal vs Signal vs RwSignal
- Use `ReadSignal<T>` when the child only reads the value.
- Use `Signal<T>` with `#[prop(into)]` for maximum flexibility — it accepts `ReadSignal`, `RwSignal`, `Memo`, and closures.
- Use `Callback<T>` for event handlers (child → parent communication).
- Only pass `RwSignal<T>` when the child **must** write to the signal.

```rust
#[component]
fn ProgressBar(
    #[prop(default = 100)]
    max: u16,
    #[prop(into)]
    progress: Signal<i32>,  // accepts ReadSignal, RwSignal, Memo, or closure
) -> impl IntoView {
    view! { <progress max=max value=progress /> }
}
```

Source: https://book.leptos.dev/view/03_components.html

### Optional and default props
- `#[prop(optional)]` — defaults to `Default::default()`
- `#[prop(default = value)]` — custom default
- `#[prop(into)]` — auto-calls `.into()` on the passed value

```rust
#[component]
fn Badge(
    #[prop(optional)]
    variant: &'static str,
    #[prop(default = false)]
    active: bool,
    #[prop(into)]
    label: Signal<String>,
) -> impl IntoView { /* ... */ }
```

Source: https://book.leptos.dev/view/03_components.html

### Document components and props
Use `///` doc comments on the component function and on each prop parameter. These render in IDE tooltips.

```rust
/// Shows progress toward a goal.
#[component]
fn ProgressBar(
    /// The maximum value of the progress bar.
    #[prop(default = 100)]
    max: u16,
    /// How much progress should be displayed.
    #[prop(into)]
    progress: Signal<i32>,
) -> impl IntoView { /* ... */ }
```

Source: https://book.leptos.dev/view/03_components.html

## Leptos Signal Types

### ReadSignal, WriteSignal, and RwSignal
`signal()` returns a `(ReadSignal<T>, WriteSignal<T>)` pair. `RwSignal::new()` creates a single reference supporting both read and write.

**Reading:** `.get()` clones the value, `.with(|v| ...)` borrows it (avoids clone), `.read()` returns a read guard.
**Writing:** `.set(val)` replaces, `.update(|v| ...)` mutates in place, `.write()` returns a mutable guard.

```rust
let (names, set_names) = signal(Vec::new());
// Prefer .with() when you only need a reference (avoids cloning)
if names.with(|n| n.is_empty()) {
    set_names.write().push("Alice".to_string());
}
```

Source: https://book.leptos.dev/reactivity/working_with_signals.html

### Memo for derived computations
Use `Memo` when a derived value is expensive to compute or read by multiple consumers. It caches the result and only recomputes when dependencies change.

```rust
let (count, set_count) = signal(1);
// Simple derived signal (closure) — recomputes on every read
let doubled = move || count.get() * 2;
// Memo — caches result, only recomputes when count changes
let doubled_memo = Memo::new(move |_| count.get() * 2);
```

Source: https://book.leptos.dev/reactivity/working_with_signals.html

### Avoid effects writing to signals
Effects that write to signals create inefficient reactive graphs and risk infinite loops. Prefer `Memo` for derived state instead of `Effect` + signal pairs.

Source: https://book.leptos.dev/reactivity/working_with_signals.html

## Leptos Control Flow

### Conditional rendering with `if/else` and `.into_any()`
Use standard Rust `if/else` or `match` in views. When branches return different HTML element types, call `.into_any()` to erase the type.

```rust
{move || match is_odd() {
    true => view! { <pre>"Odd"</pre> }.into_any(),
    false => view! { <p>"Even"</p> }.into_any(),
}}
```

Source: https://book.leptos.dev/view/06_control_flow.html

### The `<Show/>` component
Memoizes the `when` condition so it only renders/destroys children when the boolean changes. Use for expensive components; use direct `if` for lightweight text changes.

```rust
<Show
    when=move || { value.get() > 5 }
    fallback=|| view! { <Small/> }
>
    <Big/>
</Show>
```

Source: https://book.leptos.dev/view/06_control_flow.html

### Option<T> for conditional display
`Option<T>` implements `IntoView` — `Some(view)` renders, `None` renders nothing.

```rust
let message = move || is_odd().then(|| "Ding ding ding!");
view! { <p>{message}</p> }
```

Source: https://book.leptos.dev/view/06_control_flow.html

## Leptos Dynamic Lists

### The `<For/>` component
Keyed dynamic list renderer for lists that grow, shrink, or reorder. More efficient than `.iter().map().collect()` for dynamic data.

```rust
<For
    each=move || counters.get()
    key=|counter| counter.0   // stable unique ID — never use index
    children=move |(id, count)| {
        view! { <li><button on:click=move |_| *count.write() += 1>{count}</button></li> }
    }
/>
```

**When to use:** Dynamic lists where items are added/removed/reordered.
**When `.iter().map().collect()` is fine:** Static or rarely-changing lists.

Source: https://book.leptos.dev/view/04_iteration.html

## Leptos Component Composition

### Children prop
Use `Children` (= `Box<dyn FnOnce() -> AnyView>`) to accept child elements. Use `ChildrenFn` if children need to be called multiple times.

```rust
#[component]
fn Card(children: Children) -> impl IntoView {
    view! {
        <div class="card">
            {children()}
        </div>
    }
}
```

Source: https://book.leptos.dev/view/09_component_children.html

### Render props
Pass rendering functions as regular props for named slots:

```rust
#[component]
fn Layout<F, IV>(header: F, children: Children) -> impl IntoView
where
    F: Fn() -> IV,
    IV: IntoView,
{
    view! {
        <header>{header()}</header>
        <main>{children()}</main>
    }
}
```

Source: https://book.leptos.dev/view/09_component_children.html

