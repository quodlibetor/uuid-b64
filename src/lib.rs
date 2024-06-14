//! A UUID wrapper that has a base64 display and serialization
//!
//! # What?
//!
//! A newtype around UUIDs that:
//!
//! * Displays and Serializes as Base64
//!   * Specifically it is the url-safe base64 variant, *with no padding*
//!
//! ```rust
//! # extern crate uuid;
//! # extern crate uuid_b64;
//! # use uuid::Uuid;
//! # use uuid_b64::UuidB64;
//! # fn main() {
//! let known_id = Uuid::parse_str("b0c1ee86-6f46-4f1b-8d8b-7849e75dbcee").unwrap();
//! let as_b64 = UuidB64::from(known_id);
//! assert_eq!(as_b64.to_string(), "sMHuhm9GTxuNi3hJ51287g");
//!
//! let parsed_b64: UuidB64 = "sMHuhm9GTxuNi3hJ51287g".parse().unwrap();
//! assert_eq!(parsed_b64, as_b64);
//!
//! let raw_id = Uuid::new_v4();
//! assert_eq!(raw_id.to_string().len(), 36);
//! let uuidb64 = UuidB64::from(raw_id);
//! assert_eq!(uuidb64.to_string().len(), 22);
//! # }
//! ```
//!
//! `UuidB64::new` creates v4 UUIDs, because... that's what I use. I'm open to
//! hearing arguments about why this is a ridiculous decision and I should have
//! made `new` be `new_v4`.
//!
//! # Why?
//!
//! UUIDs are great:
//!
//! * They have a known size and representation, meaning that they are
//!   well-supported with an efficient representation in a wide variety of
//!   systems. Things like programming languages and databases.
//! * V4 (almost completely random) UUIDs have nice sharding properties, you
//!   can give out UUIDs willy-nilly without coordination and still be
//!   guaranteed to not have a conflict of IDs between two items across
//!   systems.
//!
//! That said, the standard *representation* for UUIDs is kind of annoying:
//!
//! * It's a *long*: 36 characters to represent 16 bytes of data!
//! * It's hard to read: it is only hexadecimal characters. The human eye needs
//!   to pay a lot of attention to be certain if any two UUIDs are the same.
//!
//! I guess that's it. Base64 is a more human-friendly representation of UUIDs:
//!
//! * It's slightly shorter: 1.375 times the size of the raw data (22
//!   characters), vs 2.25 times the size characters.
//! * Since it is case-sensitive, the *shape* of the IDs helps to distinguish
//!   between different IDs. There is also more entropy per character, so
//!   scanning to find a difference is faster.
//!
//! That said, there are drawbacks to something like this:
//!
//! * If you store it as a UUID column in a database IDs won't show up in the
//!   same way as it does in your application code, meaning you'll (A) maybe
//!   want to define a new database type, or B just expect to only ever
//!   interact with the DB via you application.
//!
//!   Conversion functions are pretty trivial, this works in postgres
//!   (inefficiently, but it's only for interactive queries so whatever):
//!
//!   ```sql
//!   CREATE FUNCTION b64uuid(encoded TEXT) RETURNS UUID
//!   AS $$
//!       BEGIN
//!           RETURN ENCODE(DECODE(REPLACE(REPLACE(
//!               encoded, '-', '+'), '_', '/') || '==', 'base64'), 'hex')::UUID;
//!       END
//!   $$ LANGUAGE plpgsql;
//!   ```
//!
//! # Usage
//!
//! Just use `UuidB64` everywhere you would use `Uuid`, and use `UuidB64::from`
//! to create one from an existing UUID.
//!
//! ## Features
//!
//! * `serde` enables serialization/deserialization via Serde.
//! * `diesel-uuid` enables integration with Diesel's UUID support, this is
//!   only tested on postgres, PRs welcome for other DBs.

#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel_derive_newtype;
#[macro_use]
extern crate error_chain;
extern crate inlinable_string;
extern crate uuid;

#[cfg(all(test, feature = "diesel-uuid"))]
#[macro_use]
extern crate diesel;
#[cfg(all(test, feature = "diesel-uuid"))]
#[cfg(all(test, feature = "serde"))]
#[macro_use]
extern crate serde_derive;
#[cfg(all(test, feature = "serde"))]
#[macro_use]
extern crate serde_json;

use std::convert::From;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use base64::display::Base64Display;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use inlinable_string::inline_string::InlineString;
use uuid::Uuid;

use crate::errors::{ErrorKind, ResultExt};

mod errors;
#[cfg(feature = "serde")]
mod serde_impl;

/// It's a Uuid that displays as Base 64
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "diesel", derive(DieselNewType))]
pub struct UuidB64(uuid::Uuid);

impl UuidB64 {
    /// Generate a new v4 Uuid
    pub fn new() -> UuidB64 {
        UuidB64(Uuid::new_v4())
    }

    /// Copy the raw UUID out
    pub fn uuid(&self) -> Uuid {
        self.0
    }

    /// Convert this to a new [`InlineString`][]
    ///
    /// `InlineString`s are stack-allocated and therefore faster than
    /// heap-allocated Strings. How much faster? Somewhat faster:
    ///
    /// ```text
    /// test uuidb64_to_inline_string                 ... bench:          49 ns/iter (+/- 20)
    /// test uuidb64_to_inline_string_new_id_per_loop ... bench:         146 ns/iter (+/- 23)
    /// test uuidb64_to_string                        ... bench:         151 ns/iter (+/- 28)
    /// test uuidb64_to_string_new_id_per_loop        ... bench:         268 ns/iter (+/- 144)
    /// ```
    ///
    /// Honestly this is unlikely to matter for your use case, but since B64
    /// UUIDs have a serialization that *does* fit within the InlineString
    /// limit (where the regular UUID representation does not) it felt like a
    /// waste to not do this. Also this is what is used for Serde, so we're
    /// zero-allocation for that.
    ///
    /// [`InlineString`]: https://docs.rs/inlinable_string/0.1.9/inlinable_string/inline_string/index.html
    pub fn to_istring(&self) -> InlineString {
        let mut buf = InlineString::from("0000000000000000000000"); // not actually zeroes
        unsafe {
            let raw_buf = buf.as_mut_slice();
            URL_SAFE_NO_PAD.encode_slice(self.0.as_bytes(), &mut raw_buf[0..22]).unwrap();
        }
        buf
    }

    /// Write the Base64-encoded UUID into the provided buffer
    ///
    /// ```
    /// # extern crate uuid;
    /// # extern crate uuid_b64;
    /// # use uuid::Uuid;
    /// # use uuid_b64::UuidB64;
    /// # fn main() {
    /// let known_id = Uuid::parse_str("b0c1ee86-6f46-4f1b-8d8b-7849e75dbcee").unwrap();
    /// let as_b64 = UuidB64::from(known_id);
    /// let mut buf = String::new();
    /// as_b64.to_buf(&mut buf);
    /// assert_eq!(&buf, "sMHuhm9GTxuNi3hJ51287g");
    /// # }
    /// ```
    pub fn to_buf(&self, buffer: &mut String) {
        URL_SAFE_NO_PAD.encode_string(self.0.as_bytes(), buffer);
    }
}

/// Parse a B64 encoded string into a UuidB64
///
/// ```rust
/// # use uuid_b64::UuidB64;
/// let parsed_b64: UuidB64 = "sMHuhm9GTxuNi3hJ51287g".parse().unwrap();
/// assert_eq!(format!("{:?}", parsed_b64), "UuidB64(sMHuhm9GTxuNi3hJ51287g)");
/// ```
impl FromStr for UuidB64 {
    type Err = errors::ErrorKind;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Don't allocated here
        let bytes = URL_SAFE_NO_PAD.decode(s)
            .chain_err(|| ErrorKind::ParseError(s.into()))?;
        let id = Uuid::from_bytes(&bytes).chain_err(|| ErrorKind::ParseError(s.into()))?;
        Ok(UuidB64(id))
    }
}

/// Right now this is just `Uuid`, but anything Uuid is comfortable with, we are
impl<T> From<T> for UuidB64
where
    T: Into<Uuid>,
{
    fn from(item: T) -> Self {
        UuidB64(item.into())
    }
}

impl Debug for UuidB64 {
    /// Same as the display formatter, but includes `UuidB64()` around it
    ///
    /// ```rust
    /// # extern crate uuid;
    /// # extern crate uuid_b64;
    /// # use uuid::Uuid;
    /// # use uuid_b64::UuidB64;
    /// # fn main() {
    /// let known_id = Uuid::parse_str("b0c1ee86-6f46-4f1b-8d8b-7849e75dbcee").unwrap();
    /// let as_b64 = UuidB64::from(known_id);
    /// assert_eq!(format!("{:?}", as_b64), "UuidB64(sMHuhm9GTxuNi3hJ51287g)");
    /// # }
    /// ```
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "UuidB64({})", self)
    }
}

impl Display for UuidB64 {
    /// Write Base64 encoding of this UUID
    ///
    /// ```rust
    /// # extern crate uuid;
    /// # extern crate uuid_b64;
    /// # use uuid::Uuid;
    /// # use uuid_b64::UuidB64;
    /// # fn main() {
    /// let known_id = Uuid::parse_str("b0c1ee86-6f46-4f1b-8d8b-7849e75dbcee").unwrap();
    /// let as_b64 = UuidB64::from(known_id);
    /// assert_eq!(format!("{}", as_b64), "sMHuhm9GTxuNi3hJ51287g");
    /// # }
    /// ```
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let wrapper = Base64Display::new(self.0.as_bytes(), &URL_SAFE_NO_PAD);
        write!(f, "{}", wrapper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_b64() {
        let id = UuidB64::new();
        let fmted = format!("{}", id);
        assert_eq!(fmted.len(), 22);
        assert_eq!(format!("UuidB64({})", fmted), format!("{:?}", id));
    }

    #[test]
    fn parse_roundtrips() {
        let original = UuidB64::new();
        let encoded = format!("{}", original);
        let parsed: UuidB64 = encoded.parse().unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn from_uuid_works() {
        let _ = UuidB64::from(Uuid::new_v4());
    }

    #[test]
    fn to_istring_works() {
        let b64 = UuidB64::from(Uuid::parse_str("b0c1ee86-6f46-4f1b-8d8b-7849e75dbcee").unwrap());
        assert_eq!(b64.to_istring(), "sMHuhm9GTxuNi3hJ51287g");

        for _ in 0..10 {
            let b64 = UuidB64::new();
            b64.to_istring();
        }
    }
}

#[cfg(all(test, feature = "diesel-uuid"))]
mod diesel_tests {
    use diesel;
    use diesel::dsl::sql;
    use diesel::pg::PgConnection;
    use diesel::prelude::*;

    use std::env;

    use super::UuidB64;

    #[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
    #[table_name = "my_entities"]
    pub struct MyEntity {
        id: UuidB64,
        val: i32,
    }

    table! {
        my_entities {
            id -> Uuid,
            val -> Integer,
        }
    }

    #[cfg(test)]
    fn setup() -> PgConnection {
        let db_url = env::var("PG_DATABASE_URL").expect("PG_DB_URL must be in the environment");
        let conn = PgConnection::establish(&db_url).unwrap();
        #[allow(deprecated)] // not present in diesel 1.0
        let setup = sql::<diesel::types::Bool>(
            "CREATE TABLE IF NOT EXISTS my_entities (
                id UUID PRIMARY KEY,
                val Int
         )",
        );
        setup.execute(&conn).expect("Can't create table");
        conn
    }

    #[test]
    fn does_roundtrip() {
        use self::my_entities::dsl::*;

        let conn = setup();

        let obj = MyEntity {
            id: UuidB64::new(),
            val: 1,
        };

        diesel::insert_into(my_entities)
            .values(&obj)
            .execute(&conn)
            .expect("Couldn't insert struct into my_entities");

        let found: Vec<MyEntity> = my_entities.load(&conn).unwrap();
        assert_eq!(found[0], obj);

        diesel::delete(my_entities.filter(id.eq(&obj.id)))
            .execute(&conn)
            .expect("Couldn't delete existing object");
    }
}
