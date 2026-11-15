# Handwritten Notes on Rust

### package organization

A rust **package** is organized like:

```text
my_package/
├── Cargo.toml
├── Cargo.lock
├── src/
├──--- main.rs
├──--- lib.rs
├──--- bin/
├──------ binary.rs
```

A package defines one or more **crates**. A crate can be a binary or a library.
- `src/main.rs` is the default binary crate.
- `src/lib.rs` is the package's library crate.
- `src/bin/` is an automatically discovered location for binaries.
- Other files and directories can hold **modules**.

The terms `main`, `lib`, and `bin` are special.

When you don't have other binaries, `cargo run` will run the program `src/main.rs` from `main()`. Otherwise, you need to specify:
- `cargo run --bin my_package` for `src/main.rs`
- `cargo run --bin binary` for `src/bin/binary.rs`

### modules and visibility

A **module** is a container to organize code: if Rust code is organized as a tree, a module is a subtree.

Declaring a module is like creating a child node. You can only declare immediate children "one-level deep". You can declare a module in two ways: inline or in a separate file. 

```rust
mod x {
    // code here
}
```

```rust
mod x;  // code exists in a separate file
```

When you declare a module like `mod x;` in `src/main.rs` or `src/lib.rs`, Rust will look for `src/x.rs` or `src/x/mod.rs`.

When you declare a module like `mod x;` in `src/abc.rs` or `src/abc/mod.rs`, Rust will look for `src/abc/x.rs` or `src/abc/x/mod.rs`.

Declaring `pub mod x;` in `src/abc.rs` makes module `x` accessible to anyone who can access `abc`.

To access modules or items within a module, you can use a path from the root. For example, the path to function `f()` in `src/abc/x.rs` is: `crate::abc::x::f`. This assumes visibility is configured properly.

Private items are accessible to the module that defined them, and their descendants. For example, assume you have `abc::p()`, `abc::x::q()`, and `abc::y::r()` with no `pub` declarations.
- From `abc`, you can access `p`, `x`, and `y`, but not `q` or `r`.
- From `x`, you can access `p`, `y`, and `q`, but not `r`.

You can also use other path prefixes:
- `self::` for the current module.
- `super::` for the parent module.
- `crate_name::` for another crate, such as an external dependency.
- `my_package::` is an example of above, where the crate is the package's library crate.

For example:
- In `abc` you can use `self::x::f`
- In `x` you can use `super::x::f`
- In a different package you can use `my_package::abc::x::f`

The term `use` brings the item name in scope. That is, `use crate::abc::x::f;` allows you to refer to `f` directly. 
- `use crate::abc::x::f as other_f;` sets an alias.

Structs and fields can also have visibility set. For example, we may have the following in `x.rs`.

```rust
struct A {}         // private struct
pub struct B {      // public struct
    pub a: String,  // public field
    b: String,      // private field    
}
```

### testing

In Rust, unit tests are commonly placed at bottom of the source file. Here is one example:

```rust
fn f() {}

#[cfg(test)]
mod tests {
    // you should know what this does now and why it works
    use super::*;
    // `f` belongs to the parent, so the `tests` module has access to it. Then, `use super::*;` brings `f` into scope.
    
    #[test]
    fn test1() {}

    #[test]
    fn test2() {}
}
```

- `#[cfg(test)]`: only compile this module when building tests
- `#[test]`: register this function as a test

Integrations tests check the *public API*, so they are put outside `src/`. This lets them access the library like an external customer.

```text
my_package/
├── Cargo.toml
├── Cargo.lock
├── src/
├── tests/
```
