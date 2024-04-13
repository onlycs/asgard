<!-- cargo-rdme start -->

# Helheim

Basically just a Display implementation, but for warnings. Works like `thiserror`. Wraps the `log` crate

## Example
```rust
use helheim::Warning;

#[derive(Warning)]
enum MyWarning {
   #[warning("Something went wrong")]
   Something,
}

let warning = MyWarning::Something;

warning.emit(); // log::warn!("Something went wrong");
```

<!-- cargo-rdme end -->
