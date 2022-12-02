mod key_serializer;

use std::fmt::Display;

use anyhow::anyhow;
use serde::{ser, Serialize};

use anyhow::Error as AnyhowError;
use anyhow::Result;

/// Error for firestore serializer
#[derive(Debug)]
pub struct Error {
    inner: AnyhowError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error {
            inner: anyhow!(msg.to_string()),
        }
    }
}

impl std::error::Error for Error {}

/// Serializer to create Firestore documents out of structs
pub struct Serializer {
    // This string starts empty and JSON is appended as values are serialized.
    output: String,
}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.output += r#""booleanValue": "#;
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.serialize_i64(i64::from(v))
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.output += r#""integerValue": "#;
        self.output += format!("\"{}\"", &v).as_str();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.output += r#""integerValue": "#;
        self.output += format!("\"{}\"", &v).as_str();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        self.output += r#""doubleValue": "#;
        self.output += &v.to_string();
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<(), Error> {
        if time::OffsetDateTime::parse(v, &time::format_description::well_known::Rfc3339).is_ok() {
            self.output += r#""timestampValue": "#;
        } else {
            self.output += r#""stringValue": "#;
        }
        let escaped_string = serde_json::to_string(v);
        match escaped_string {
            Ok(es) => self.output += es.as_str(),
            Err(e) => return Err(Error { inner: anyhow!(e) }),
        };
        Ok(())
    }

    // Serialize a byte array as a base64 string
    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.output += r#""bytesValue": "#;
        self.output += "\"";
        self.output += &base64::encode(v);
        self.output += "\"";
        Ok(())
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<(), Error> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<(), Error> {
        self.output += r#""nullValue": "#;
        self.output += "null";
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.output += r#""mapValue": "#;
        self.output += "{\"fields\": {\"";
        self.output += variant;
        self.output += "\": ";
        value.serialize(&mut *self)?;
        self.output += "}}";
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        self.output += r#""arrayValue": {"values": ["#;
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        self.output += r#""mapValue": "#;
        self.output += "{\"fields\": {\"";
        self.output += variant;
        self.output += r#"": "arrayValue": {"values": ["#;
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        self.output += r#""mapValue": "#;
        self.output += "{\"fields\": {";
        Ok(self)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        self.serialize_map(Some(len))
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        self.output += r#""mapValue": "#;
        self.output += "{\"fields\": {";
        self.output += variant;
        self.output += r#""mapValue": "#;
        self.output += "{\"fields\": {";
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        self.output += "{";
        value.serialize(&mut **self)?;
        self.output += "}";
        Ok(())
    }

    // Close the sequence.
    fn end(self) -> Result<(), Error> {
        self.output += "]}";
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        self.output += "{";
        value.serialize(&mut **self)?;
        self.output += "}";
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.output += "]}";
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        self.output += "{";
        value.serialize(&mut **self)?;
        self.output += "}";
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.output += "]}";
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        self.output += "{";
        value.serialize(&mut **self)?;
        self.output += "}";
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.output += "]}}}";
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        let mut special_serializer = key_serializer::KeySerializer::new();

        key.serialize(&mut special_serializer)?;
        self.output += special_serializer.get_output().as_str();
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.output += ": {";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), Error> {
        self.output += "}}}";
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        self.output += format!("\"{key}\"").as_str();
        self.output += ": {";
        value.serialize(&mut **self)?;
        self.output += "}";
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.output += "}}";
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        self.output += format!("\"{key}\"").as_str();
        self.output += ": {";
        value.serialize(&mut **self)?;
        self.output += "}";
        Ok(())
    }

    fn end(self) -> Result<(), Error> {
        self.output += "}}}}";
        Ok(())
    }
}

/// Serde function that can be used to serialize an OffsetDateTime
pub fn serialize_datetime<S>(v: &time::OffsetDateTime, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(
        v.format(&time::format_description::well_known::Rfc3339)
            .unwrap()
            .as_str(),
    )
}

/// Convert a serialized string to a Firestore document.
pub fn to_document(doc: String) -> String {
    if let Some(s) = doc.strip_prefix("\"mapValue\": ") {
        s.to_owned()
    } else {
        doc
    }
}
