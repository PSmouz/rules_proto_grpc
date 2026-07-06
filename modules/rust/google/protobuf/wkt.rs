use std::collections::HashMap;

use prost::bytes::Bytes;
use serde::de::{Error as DeError, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Empty {}

impl Serialize for Empty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Object(map) if map.is_empty() => Ok(Empty {}),
            _ => Err(D::Error::custom(
                "google.protobuf.Empty must be an empty JSON object",
            )),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Timestamp {
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if !(0..1_000_000_000).contains(&self.nanos) {
            return Err(serde::ser::Error::custom("Timestamp nanos out of range"));
        }
        serializer.serialize_str(&format_timestamp(self.seconds, self.nanos))
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TimestampVisitor;

        impl<'de> Visitor<'de> for TimestampVisitor {
            type Value = Timestamp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an RFC 3339 timestamp string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Timestamp, E>
            where
                E: DeError,
            {
                parse_timestamp(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(TimestampVisitor)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Duration {
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.nanos <= -1_000_000_000 || self.nanos >= 1_000_000_000 {
            return Err(serde::ser::Error::custom("Duration nanos out of range"));
        }
        if self.seconds != 0 && self.nanos != 0 && (self.seconds < 0) != (self.nanos < 0) {
            return Err(serde::ser::Error::custom("Duration has inconsistent signs"));
        }
        serializer.serialize_str(&format_duration(self.seconds, self.nanos))
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DurationVisitor;

        impl<'de> Visitor<'de> for DurationVisitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a protobuf duration string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Duration, E>
            where
                E: DeError,
            {
                parse_duration(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(DurationVisitor)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Any {
    #[prost(string, tag = "1")]
    pub type_url: String,
    #[prost(bytes = "bytes", tag = "2")]
    pub value: Bytes,
}

impl Serialize for Any {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(if self.value.is_empty() { 1 } else { 2 }))?;
        map.serialize_entry("@type", &self.type_url)?;
        if !self.value.is_empty() {
            map.serialize_entry("value", &encode_base64(&self.value))?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Any {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut object = expect_object(serde_json::Value::deserialize(deserializer)?)
            .map_err(D::Error::custom)?;
        let type_url = take_string(&mut object, "@type")
            .or_else(|| take_string(&mut object, "typeUrl"))
            .unwrap_or_default();
        let value = match object.remove("value") {
            Some(serde_json::Value::String(value)) => {
                decode_base64(&value).map_err(D::Error::custom)?
            }
            Some(_) => return Err(D::Error::custom("Any.value must be a base64 string")),
            None => Bytes::new(),
        };
        if !object.is_empty() {
            return Err(D::Error::custom("unexpected fields in google.protobuf.Any"));
        }
        Ok(Any { type_url, value })
    }
}

#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct FieldMask {
    #[prost(string, repeated, tag = "1")]
    pub paths: Vec<String>,
}

impl Serialize for FieldMask {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &self
                .paths
                .iter()
                .map(|path| snake_to_lower_camel(path))
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

impl<'de> Deserialize<'de> for FieldMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FieldMaskVisitor;

        impl<'de> Visitor<'de> for FieldMaskVisitor {
            type Value = FieldMask;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a comma-separated field mask string")
            }

            fn visit_str<E>(self, value: &str) -> Result<FieldMask, E>
            where
                E: DeError,
            {
                Ok(FieldMask {
                    paths: if value.is_empty() {
                        Vec::new()
                    } else {
                        value.split(',').map(lower_camel_to_snake).collect()
                    },
                })
            }
        }

        deserializer.deserialize_str(FieldMaskVisitor)
    }
}

#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct DoubleValue {
    #[prost(double, tag = "1")]
    pub value: f64,
}

#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct FloatValue {
    #[prost(float, tag = "1")]
    pub value: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Int64Value {
    #[prost(int64, tag = "1")]
    pub value: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]
pub struct UInt64Value {
    #[prost(uint64, tag = "1")]
    pub value: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Int32Value {
    #[prost(int32, tag = "1")]
    pub value: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]
pub struct UInt32Value {
    #[prost(uint32, tag = "1")]
    pub value: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, ::prost::Message)]
pub struct BoolValue {
    #[prost(bool, tag = "1")]
    pub value: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct StringValue {
    #[prost(string, tag = "1")]
    pub value: String,
}

#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct BytesValue {
    #[prost(bytes = "bytes", tag = "1")]
    pub value: Bytes,
}

macro_rules! scalar_wrapper_serde {
    ($typ:ty, $value:ty) => {
        impl Serialize for $typ {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.value.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $typ {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Ok(Self {
                    value: <$value>::deserialize(deserializer)?,
                })
            }
        }

        impl From<$value> for $typ {
            fn from(value: $value) -> Self {
                Self { value }
            }
        }
    };
}

scalar_wrapper_serde!(DoubleValue, f64);
scalar_wrapper_serde!(FloatValue, f32);
scalar_wrapper_serde!(Int32Value, i32);
scalar_wrapper_serde!(UInt32Value, u32);
scalar_wrapper_serde!(BoolValue, bool);
scalar_wrapper_serde!(StringValue, String);

impl Serialize for Int64Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value.to_string())
    }
}

impl<'de> Deserialize<'de> for Int64Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_i64_json_value(deserializer).map(|value| Self { value })
    }
}

impl From<i64> for Int64Value {
    fn from(value: i64) -> Self {
        Self { value }
    }
}

impl Serialize for UInt64Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value.to_string())
    }
}

impl<'de> Deserialize<'de> for UInt64Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_u64_json_value(deserializer).map(|value| Self { value })
    }
}

impl From<u64> for UInt64Value {
    fn from(value: u64) -> Self {
        Self { value }
    }
}

impl Serialize for BytesValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&encode_base64(&self.value))
    }
}

impl<'de> Deserialize<'de> for BytesValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self {
            value: decode_base64(&value).map_err(D::Error::custom)?,
        })
    }
}

impl From<Bytes> for BytesValue {
    fn from(value: Bytes) -> Self {
        Self { value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum NullValue {
    NullValue = 0,
}

impl NullValue {
    pub fn as_str_name(&self) -> &'static str {
        "NULL_VALUE"
    }

    pub fn from_str_name(value: &str) -> Option<Self> {
        match value {
            "NULL_VALUE" => Some(Self::NullValue),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(oneof = "value::Kind", tags = "1, 2, 3, 4, 5, 6")]
    pub kind: Option<value::Kind>,
}

pub mod value {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(enumeration = "super::NullValue", tag = "1")]
        NullValue(i32),
        #[prost(double, tag = "2")]
        NumberValue(f64),
        #[prost(string, tag = "3")]
        StringValue(String),
        #[prost(bool, tag = "4")]
        BoolValue(bool),
        #[prost(message, tag = "5")]
        StructValue(super::Struct),
        #[prost(message, tag = "6")]
        ListValue(super::ListValue),
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.kind {
            Some(value::Kind::NullValue(_)) | None => serializer.serialize_unit(),
            Some(value::Kind::NumberValue(value)) => serializer.serialize_f64(*value),
            Some(value::Kind::StringValue(value)) => serializer.serialize_str(value),
            Some(value::Kind::BoolValue(value)) => serializer.serialize_bool(*value),
            Some(value::Kind::StructValue(value)) => value.serialize(serializer),
            Some(value::Kind::ListValue(value)) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        json_value_to_value(serde_json::Value::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self {
            kind: Some(value::Kind::BoolValue(value)),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self {
            kind: Some(value::Kind::NumberValue(value)),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            kind: Some(value::Kind::StringValue(value)),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl From<value::Kind> for Value {
    fn from(kind: value::Kind) -> Self {
        Self { kind: Some(kind) }
    }
}

impl From<Struct> for Value {
    fn from(value: Struct) -> Self {
        Self {
            kind: Some(value::Kind::StructValue(value)),
        }
    }
}

impl From<ListValue> for Value {
    fn from(value: ListValue) -> Self {
        Self {
            kind: Some(value::Kind::ListValue(value)),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for Value
where
    T: Into<Value>,
{
    fn from(values: [T; N]) -> Self {
        ListValue::from(values).into()
    }
}

impl From<Vec<Value>> for Value {
    fn from(values: Vec<Value>) -> Self {
        ListValue { values }.into()
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(fields: HashMap<String, Value>) -> Self {
        Struct { fields }.into()
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Struct {
    #[prost(map = "string, message", tag = "1")]
    pub fields: HashMap<String, Value>,
}

impl Serialize for Struct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.fields.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Struct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            fields: HashMap::<String, Value>::deserialize(deserializer)?,
        })
    }
}

impl From<HashMap<String, Value>> for Struct {
    fn from(fields: HashMap<String, Value>) -> Self {
        Self { fields }
    }
}

impl FromIterator<(String, Value)> for Struct {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self {
            fields: iter.into_iter().collect(),
        }
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListValue {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<Value>,
}

impl Serialize for ListValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.values.len()))?;
        for value in &self.values {
            seq.serialize_element(value)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for ListValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            values: Vec::<Value>::deserialize(deserializer)?,
        })
    }
}

impl<T> From<Vec<T>> for ListValue
where
    T: Into<Value>,
{
    fn from(values: Vec<T>) -> Self {
        Self {
            values: values.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T, const N: usize> From<[T; N]> for ListValue
where
    T: Into<Value>,
{
    fn from(values: [T; N]) -> Self {
        Self {
            values: values.into_iter().map(Into::into).collect(),
        }
    }
}

impl<T> FromIterator<T> for ListValue
where
    T: Into<Value>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            values: iter.into_iter().map(Into::into).collect(),
        }
    }
}

fn format_timestamp(seconds: i64, nanos: i32) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    let mut formatted = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
    if nanos != 0 {
        formatted.push_str(&format_fractional_nanos(nanos));
    }
    formatted.push_str("+00:00");
    formatted
}

fn parse_timestamp(value: &str) -> Result<Timestamp, String> {
    let (value, offset_seconds) = split_timestamp_timezone(value)?;
    let (date, time) = value
        .split_once('T')
        .ok_or_else(|| "timestamp must contain 'T'".to_owned())?;
    let mut date_parts = date.split('-');
    let year: i32 = parse_next(&mut date_parts, "year")?;
    let month: u32 = parse_next(&mut date_parts, "month")?;
    let day: u32 = parse_next(&mut date_parts, "day")?;
    if date_parts.next().is_some() {
        return Err("timestamp date has too many components".to_owned());
    }

    let (time, nanos) = parse_fraction(time)?;
    let mut time_parts = time.split(':');
    let hour: i64 = parse_next(&mut time_parts, "hour")?;
    let minute: i64 = parse_next(&mut time_parts, "minute")?;
    let second: i64 = parse_next(&mut time_parts, "second")?;
    if time_parts.next().is_some() {
        return Err("timestamp time has too many components".to_owned());
    }
    if hour > 23 || minute > 59 || second > 59 {
        return Err("timestamp time component out of range".to_owned());
    }

    let days = days_from_civil(year, month, day);
    Ok(Timestamp {
        seconds: days * 86_400 + hour * 3_600 + minute * 60 + second - offset_seconds,
        nanos,
    })
}

fn split_timestamp_timezone(value: &str) -> Result<(&str, i64), String> {
    if let Some(value) = value.strip_suffix('Z') {
        return Ok((value, 0));
    }

    let time_start = value
        .find('T')
        .ok_or_else(|| "timestamp must contain 'T'".to_owned())?
        + 1;
    let timezone_start = value[time_start..]
        .rfind(|ch| ch == '+' || ch == '-')
        .map(|index| time_start + index)
        .ok_or_else(|| "timestamp must contain a timezone offset".to_owned())?;
    let (timestamp, offset) = value.split_at(timezone_start);
    let sign = if offset.starts_with('+') { 1 } else { -1 };
    let offset = &offset[1..];
    let (hours, minutes) = offset
        .split_once(':')
        .ok_or_else(|| "timestamp timezone offset must use HH:MM".to_owned())?;
    let hours: i64 = hours
        .parse()
        .map_err(|_| "invalid timezone offset hours".to_owned())?;
    let minutes: i64 = minutes
        .parse()
        .map_err(|_| "invalid timezone offset minutes".to_owned())?;
    if hours > 23 || minutes > 59 {
        return Err("timestamp timezone offset out of range".to_owned());
    }

    Ok((timestamp, sign * (hours * 3_600 + minutes * 60)))
}

fn format_duration(seconds: i64, nanos: i32) -> String {
    let negative = seconds < 0 || nanos < 0;
    let seconds = seconds.abs();
    let nanos = nanos.abs();
    let mut formatted = if negative {
        format!("-{seconds}")
    } else {
        seconds.to_string()
    };
    if nanos != 0 {
        formatted.push_str(&format_fractional_nanos(nanos));
    }
    formatted.push('s');
    formatted
}

fn parse_duration(value: &str) -> Result<Duration, String> {
    let value = value
        .strip_suffix('s')
        .ok_or_else(|| "duration must end with 's'".to_owned())?;
    let (negative, value) = match value.strip_prefix('-') {
        Some(value) => (true, value),
        None => (false, value),
    };
    let (seconds, nanos) = parse_fraction(value)?;
    let seconds: i64 = seconds
        .parse()
        .map_err(|_| "invalid duration seconds".to_owned())?;
    Ok(Duration {
        seconds: if negative { -seconds } else { seconds },
        nanos: if negative { -nanos } else { nanos },
    })
}

fn format_fractional_nanos(nanos: i32) -> String {
    if nanos % 1_000_000 == 0 {
        format!(".{:03}", nanos / 1_000_000)
    } else if nanos % 1_000 == 0 {
        format!(".{:06}", nanos / 1_000)
    } else {
        format!(".{nanos:09}")
    }
}

fn parse_fraction(value: &str) -> Result<(&str, i32), String> {
    let Some((whole, fraction)) = value.split_once('.') else {
        return Ok((value, 0));
    };
    if fraction.is_empty()
        || fraction.len() > 9
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("invalid fractional nanoseconds".to_owned());
    }
    let mut nanos: i32 = fraction
        .parse()
        .map_err(|_| "invalid fractional nanoseconds".to_owned())?;
    for _ in fraction.len()..9 {
        nanos *= 10;
    }
    Ok((whole, nanos))
}

fn parse_next<'a, T>(parts: &mut impl Iterator<Item = &'a str>, name: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    parts
        .next()
        .ok_or_else(|| format!("missing {name}"))?
        .parse()
        .map_err(|_| format!("invalid {name}"))
}

fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year, month as u32, day as u32)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = month as i64;
    let day = day as i64;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn snake_to_lower_camel(path: &str) -> String {
    let mut result = String::with_capacity(path.len());
    let mut uppercase_next = false;
    for ch in path.chars() {
        if ch == '_' {
            uppercase_next = true;
        } else if uppercase_next {
            result.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn lower_camel_to_snake(path: &str) -> String {
    let mut result = String::with_capacity(path.len());
    for ch in path.chars() {
        if ch.is_ascii_uppercase() {
            result.push('_');
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn json_value_to_value(value: serde_json::Value) -> Result<Value, String> {
    Ok(match value {
        serde_json::Value::Null => value::Kind::NullValue(NullValue::NullValue as i32).into(),
        serde_json::Value::Bool(value) => value.into(),
        serde_json::Value::Number(value) => value
            .as_f64()
            .ok_or_else(|| "JSON number cannot be represented as f64".to_owned())?
            .into(),
        serde_json::Value::String(value) => value.into(),
        serde_json::Value::Array(values) => ListValue {
            values: values
                .into_iter()
                .map(json_value_to_value)
                .collect::<Result<Vec<_>, _>>()?,
        }
        .into(),
        serde_json::Value::Object(fields) => Struct {
            fields: fields
                .into_iter()
                .map(|(key, value)| Ok((key, json_value_to_value(value)?)))
                .collect::<Result<HashMap<_, _>, String>>()?,
        }
        .into(),
    })
}

fn expect_object(
    value: serde_json::Value,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    match value {
        serde_json::Value::Object(object) => Ok(object),
        _ => Err("expected JSON object".to_owned()),
    }
}

fn take_string(
    object: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    match object.remove(key) {
        Some(serde_json::Value::String(value)) => Some(value),
        Some(value) => {
            object.insert(key.to_owned(), value);
            None
        }
        None => None,
    }
}

fn deserialize_i64_json_value<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(value) => value.parse().map_err(D::Error::custom),
        serde_json::Value::Number(value) => value
            .as_i64()
            .ok_or_else(|| D::Error::custom("expected an int64 JSON value")),
        _ => Err(D::Error::custom("expected an int64 JSON string or number")),
    }
}

fn deserialize_u64_json_value<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(value) => value.parse().map_err(D::Error::custom),
        serde_json::Value::Number(value) => value
            .as_u64()
            .ok_or_else(|| D::Error::custom("expected a uint64 JSON value")),
        _ => Err(D::Error::custom("expected a uint64 JSON string or number")),
    }
}

fn encode_base64(value: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut encoded = String::with_capacity(value.len().div_ceil(3) * 4);
    for chunk in value.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn decode_base64(value: &str) -> Result<Bytes, String> {
    let mut sextets = Vec::with_capacity(value.len());
    let mut padding = 0usize;

    for byte in value.bytes() {
        let value = match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' | b'-' => Some(62),
            b'/' | b'_' => Some(63),
            b'=' => {
                padding += 1;
                None
            }
            _ => return Err("invalid base64 character".to_owned()),
        };

        match value {
            Some(value) if padding == 0 => sextets.push(Some(value)),
            Some(_) => return Err("base64 data found after padding".to_owned()),
            None => sextets.push(None),
        }
    }

    let remainder = sextets.len() % 4;
    if remainder == 1 {
        return Err("invalid base64 length".to_owned());
    }
    if remainder != 0 {
        sextets.extend(std::iter::repeat(None).take(4 - remainder));
    }

    let mut decoded = Vec::with_capacity(sextets.len() / 4 * 3);
    for chunk in sextets.chunks(4) {
        let a = chunk[0].ok_or_else(|| "invalid base64 padding".to_owned())?;
        let b = chunk[1].ok_or_else(|| "invalid base64 padding".to_owned())?;

        decoded.push((a << 2) | (b >> 4));

        match (chunk[2], chunk[3]) {
            (Some(c), Some(d)) => {
                decoded.push(((b & 0b0000_1111) << 4) | (c >> 2));
                decoded.push(((c & 0b0000_0011) << 6) | d);
            }
            (Some(c), None) => {
                decoded.push(((b & 0b0000_1111) << 4) | (c >> 2));
            }
            (None, None) => {}
            (None, Some(_)) => return Err("invalid base64 padding".to_owned()),
        }
    }

    Ok(Bytes::from(decoded))
}
