//! A serializer specifically for serializing string-only object keys

use serde::{ser, Serialize};

use anyhow::{anyhow, Result};

pub struct KeySerializer {
    // This string starts empty and JSON is appended as values are serialized.
    output: String,
}

impl KeySerializer {
    pub fn get_output(&self) -> String {
        self.output.clone()
    }
    pub fn new() -> Self {
        KeySerializer {
            output: String::new(),
        }
    }
}

impl Default for KeySerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ser::Serializer for &'a mut KeySerializer {
    type Ok = ();
    type Error = super::Error;
    type SerializeSeq = &'a mut super::Serializer;
    type SerializeTuple = &'a mut super::Serializer;
    type SerializeTupleStruct = &'a mut super::Serializer;
    type SerializeTupleVariant = &'a mut super::Serializer;
    type SerializeMap = &'a mut super::Serializer;
    type SerializeStruct = &'a mut super::Serializer;
    type SerializeStructVariant = &'a mut super::Serializer;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output += format!("\"{v}\"").as_str();
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(super::Error {
            inner: anyhow!("object keys must be strings"),
        })
    }
}
