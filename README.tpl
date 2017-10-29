# {{crate}}

{{readme}}

# Contributing

## Testing

Most tests are standard: `cargo test` or `cargo test --features serde`, but if
you want to test the diesel integration then we need a running postgres
instance. Assuming that you have docker running locally and are in bash you can
do `./run-tests.sh` to execute all tests.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
