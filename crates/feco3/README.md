# FECo3

A Rust library for parsing .FEC files.

This crate holds the core parsing logic. It is intended to be extended.

For example, we have [Python bindings](https://github.com/NickCrews/feco3).
You could add a bindings for other languages if you wanted,
or you could customize the parsing at the rust level, for example
adding a new input reader or output writer.

## Rust Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
feco3 = "VERSION"
```

Then:

```rust;

fn main() {
    let fec = feco3::FecFile::from_path("path/to/file.fec")
    println!("{:?}", fec);
}
```
