# uuid-b64

[![Push](https://github.com/quodlibetor/uuid-b64/actions/workflows/push.yml/badge.svg)](https://github.com/quodlibetor/uuid-b64/actions/workflows/push.yml)

A UUID wrapper that has a base64 display and serialization

## What?

A newtype around UUIDs that:

* Displays and Serializes as Base64
  * Specifically it is the url-safe base64 variant, *with no padding*

```rust
let known_id = Uuid::parse_str("b0c1ee86-6f46-4f1b-8d8b-7849e75dbcee").unwrap();
let as_b64 = UuidB64::from(known_id);
assert_eq!(as_b64.to_string(), "sMHuhm9GTxuNi3hJ51287g");

let parsed_b64: UuidB64 = "sMHuhm9GTxuNi3hJ51287g".parse().unwrap();
assert_eq!(parsed_b64, as_b64);

let raw_id = Uuid::new_v4();
assert_eq!(raw_id.to_string().len(), 36);
let uuidb64 = UuidB64::from(raw_id);
assert_eq!(uuidb64.to_string().len(), 22);
```

`UuidB64::new` creates v4 UUIDs, because... that's what I use. I'm open to
hearing arguments about why this is a ridiculous decision and I should have
made `new` be `new_v4`.

## Why?

UUIDs are great:

* They have a known size and representation, meaning that they are
  well-supported with an efficient representation in a wide variety of
  systems. Things like programming languages and databases.
* V4 (almost completely random) UUIDs have nice sharding properties, you
  can give out UUIDs willy-nilly without coordination and still be
  guaranteed to not have a conflict of IDs between two items across
  systems.

That said, the standard *representation* for UUIDs is kind of annoying:

* It's a *long*: 36 characters to represent 16 bytes of data!
* It's hard to read: it is only hexadecimal characters. The human eye needs
  to pay a lot of attention to be certain if any two UUIDs are the same.

I guess that's it. Base64 is a more human-friendly representation of UUIDs:

* It's slightly shorter: 1.375 times the size of the raw data (22
  characters), vs 2.25 times the size characters.
* Since it is case-sensitive, the *shape* of the IDs helps to distinguish
  between different IDs. There is also more entropy per character, so
  scanning to find a difference is faster.

That said, there are drawbacks to something like this:

* If you store it as a UUID column in a database IDs won't show up in the
  same way as it does in your application code, meaning you'll (A) maybe
  want to define a new database type, or B just expect to only ever
  interact with the DB via you application.

  Conversion functions are pretty trivial, this works in postgres
  (inefficiently, but it's only for interactive queries so whatever):

  ```sql
  CREATE FUNCTION b64uuid(encoded TEXT) RETURNS UUID
  AS $$
      BEGIN
          RETURN ENCODE(DECODE(REPLACE(REPLACE(
              encoded, '-', '+'), '_', '/') || '==', 'base64'), 'hex')::UUID;
      END
  $$ LANGUAGE plpgsql;
  ```

## Usage

Just use `UuidB64` everywhere you would use `Uuid`, and use `UuidB64::from`
to create one from an existing UUID.

### Features

* `serde` enables serialization/deserialization via Serde.
* `diesel-uuid` enables integration with Diesel's UUID support, this is
  only tested on postgres, PRs welcome for other DBs.

# Contributing

## Testing

Most tests are standard: `cargo test` or `cargo test --features serde`, but if
you want to test the diesel integration (the `diesel-uuid` feature) then we
need a running postgres instance. Assuming that you have docker running locally
and are in bash you can do `./run-tests.sh` to execute all tests.

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
