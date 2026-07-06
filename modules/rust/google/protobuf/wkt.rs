#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct Any {
    #[prost(string, tag = "1")]
    pub type_url: String,
    #[prost(bytes = "bytes", tag = "2")]
    pub value: ::prost::bytes::Bytes,
}

impl serde::Serialize for Any {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use pbjson::private::base64::engine::Engine;
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(if self.value.is_empty() { 1 } else { 2 }))?;
        map.serialize_entry("@type", &self.type_url)?;
        if !self.value.is_empty() {
            let value =
                pbjson::private::base64::engine::general_purpose::STANDARD.encode(&self.value);
            map.serialize_entry("value", &value)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Any {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["@type", "typeUrl", "value"];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TypeUrl,
            Value,
        }

        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", FIELDS)
                    }

                    fn visit_str<E>(self, value: &str) -> Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "@type" | "typeUrl" => Ok(GeneratedField::TypeUrl),
                            "value" => Ok(GeneratedField::Value),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Any;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.protobuf.Any")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Any, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut type_url = None;
                let mut value = None;
                while let Some(field) = map.next_key()? {
                    match field {
                        GeneratedField::TypeUrl => {
                            if type_url.is_some() {
                                return Err(serde::de::Error::duplicate_field("@type"));
                            }
                            type_url = Some(map.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value = Some(
                                map.next_value::<pbjson::private::BytesDeserialize<
                                    ::prost::bytes::Bytes,
                                >>()?
                                .0,
                            );
                        }
                    }
                }

                Ok(Any {
                    type_url: type_url.unwrap_or_default(),
                    value: value.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_struct("google.protobuf.Any", FIELDS, GeneratedVisitor)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, ::prost::Message)]
pub struct FieldMask {
    #[prost(string, repeated, tag = "1")]
    pub paths: Vec<String>,
}

impl serde::Serialize for FieldMask {
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

impl<'de> serde::Deserialize<'de> for FieldMask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FieldMaskVisitor;

        impl<'de> serde::de::Visitor<'de> for FieldMaskVisitor {
            type Value = FieldMask;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a comma-separated field mask string")
            }

            fn visit_str<E>(self, value: &str) -> Result<FieldMask, E>
            where
                E: serde::de::Error,
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
