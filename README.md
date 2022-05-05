# Jayson

## Introduction

Jayson is a heavily modified version of [miniserde](https://github.com/dtolnay/miniserde), with
only the deserialization part. The goal is to be a drop in replacement for serde for deserializing
json payloads and provide customizable error types that are tied to the type that the json is
being deserilialized into rather than to the deserializer's error type, like serde does. This
allows to return custom validation, rather than relying on serde's deserialization errors.

## Example

### `VisitorError` trait

The `VisitorError` is a trait that must be implemented by your error types.

```rust

struct MyError;

pub trait VisitorError: 'static {
    fn unexpected(s: &str) -> Self {
	    MyError
    }

    fn format_error(line: usize, pos: usize, msg: &str) -> Self {
	    MyError
    }

    fn missing_field(field: &str) -> Self {
	    MyError
    }
}
```

### Implementing deserialize for a custom type
```rust
miniserde::make_place!(Place);

struct Name(String);

impl Deserialize<MyError> for Name {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<MyError> {
        impl Visitor<MyError> for Place<Name> {
            fn string(&mut self, s: &str) -> Result<(), MyError> {
                if !s.chars().all(|c| c.is_ascii_alphanumeric()) {
                    Err(Error::InvalidName(s.to_string()))
                } else {
                    self.out.replace(Name(s.to_string()));
                    Ok(())
                }
            }
        }

        Place::new(out)
    }
}
```

### Using macros

```rust
#[derive(Deserialize)]
#[serde(error = "MyError")]
struct User {
	name: Name,
}
```

## Features

- [x] Derive macro for structs
- [ ] Derive macro for enums
- [x] rename all rule (camel case)
- [x] rename rule
- [ ] flatten
- [ ] Actix web extractor
- [ ] Documentation

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
