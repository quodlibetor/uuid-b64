extern crate serde;

use std::fmt::{Formatter, Result as FmtResult};

use self::serde::ser::{Serialize, Serializer};
use self::serde::de::{self, Deserialize, Deserializer, Visitor};

use super::UuidB64;

impl Serialize for UuidB64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_istring())
    }
}

impl<'de> Deserialize<'de> for UuidB64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(UuidB64Visitor)
    }
}

struct UuidB64Visitor;

impl<'de> Visitor<'de> for UuidB64Visitor {
    type Value = UuidB64;

    fn expecting(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "a URL-safe Base64-encoded string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(s.parse().map_err(de::Error::custom)?)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use UuidB64;


    #[test]
    fn ser_de() {
        let uuid = Uuid::from_fields(0xff, 2, 3, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        let my_id = UuidB64::from(uuid);

        let json = json!({ "myid": my_id }).to_string();

        assert_eq!(json, r#"{"myid":"AAAA_wACAAMBAgMEBQYHCA"}"#);

        #[derive(Deserialize)]
        struct TestThing {
            myid: UuidB64,
        }

        let mything: TestThing = ::serde_json::from_str(&json).unwrap();

        assert_eq!(mything.myid, my_id);
    }
}
