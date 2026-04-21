# Reference — Verified patterns from official documentation

This file is built incrementally during refactoring. Every entry MUST include:
1. The rule/pattern
2. A code example
3. The source URL from official documentation

**Do not add anything here that isn't verified from an official source.**

---

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

