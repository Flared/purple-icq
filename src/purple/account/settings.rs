use super::Account;
use serde::{de, ser, Serialize};
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),
    UnsupportedType(&'static str),
    MissingKey,
    PurpleFailed,
}
impl std::error::Error for Error {}

impl ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::PurpleFailed => formatter.write_str("libpurple call failed"),
            Error::MissingKey => formatter.write_str("Key unset for value"),
            Error::UnsupportedType(t) => write!(formatter, "type '{}' is not supported", t),
        }
    }
}

pub struct Serializer<'a> {
    current_key: Result<&'a str>,
    account: &'a Account,
}

pub fn to_account<T>(account: &Account, value: &T) -> Result<()>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        account,
        current_key: Err(Error::MissingKey),
    };
    value.serialize(&mut serializer)
}

impl<'a> Serializer<'a> {
    fn serialize_int(&self, v: i32) -> Result<()> {
        match &self.current_key {
            Ok(k) => {
                self.account.set_int(k, v);
                Ok(())
            }
            Err(e) => Err(e.clone()),
        }
    }
    fn serialize_string(&self, v: &str) -> Result<()> {
        match &self.current_key {
            Ok(k) => {
                self.account.set_string(k, v);
                Ok(())
            }
            Err(e) => Err(e.clone()),
        }
    }
    fn remove_settings(&self) -> Result<()> {
        match &self.current_key {
            Ok(k) => {
                self.account.remove_setting(k);
                Ok(())
            }
            Err(e) => Err(e.clone()),
        }
    }
}

impl<'a, 'b> ser::Serializer for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        match &self.current_key {
            Ok(k) => {
                self.account.set_bool(k, v);
                Ok(())
            }
            Err(e) => Err(e.clone()),
        }
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_int(v as i32)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_int(v as i32)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_int(v)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("i64"))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_int(v as i32)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_int(v as i32)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_int(v as i32)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("u64"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("f64"))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_string(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.serialize_string(v)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("bytes"))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.remove_settings()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("unit"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::UnsupportedType("unit_struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok> {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::UnsupportedType("seq"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedType("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType("tuple_struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType("tuple_variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedType("map"))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType("struct_variant"))
    }
}

impl<'a, 'b> ser::SerializeSeq for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> ser::SerializeTuple for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> ser::SerializeTupleStruct for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> ser::SerializeTupleVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> ser::SerializeMap for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<()> {
        unreachable!()
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl<'a, 'b> ser::SerializeStruct for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        // Shouldn't happen.
        if self.current_key.is_ok() {
            return Err(Error::Message("Key already set".into()));
        }

        self.current_key = Ok(key);
        let value = value.serialize(&mut **self);
        self.current_key = Err(Error::MissingKey);
        value
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, 'b> ser::SerializeStructVariant for &'a mut Serializer<'b> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}
