use std::collections::HashMap;

use google_protobuf::google::protobuf::{
    value, Any, BoolValue, BytesValue, DoubleValue, Duration, Empty, FieldMask, FloatValue,
    Int32Value, Int64Value, ListValue, StringValue, Struct, Timestamp, UInt32Value, UInt64Value,
    Value,
};
use proto_runtime::{prost::bytes::Bytes, serde_json};

#[test]
fn timestamp_uses_canonical_protojson_string() {
    let timestamp = Timestamp {
        seconds: 1_767_322_445,
        nanos: 0,
    };

    assert_eq!(
        serde_json::to_string(&timestamp).unwrap(),
        "\"2026-01-02T02:54:05+00:00\"",
    );
}

#[test]
fn duration_uses_canonical_protojson_string() {
    let duration = Duration {
        seconds: 3,
        nanos: 500_000_000,
    };

    assert_eq!(serde_json::to_string(&duration).unwrap(), "\"3.500s\"");
}

#[test]
fn wrapper_types_serialize_as_bare_scalars() {
    assert_eq!(
        serde_json::to_string(&BoolValue { value: true }).unwrap(),
        "true"
    );
    assert_eq!(
        serde_json::to_string(&StringValue {
            value: "hello".to_owned(),
        })
        .unwrap(),
        "\"hello\"",
    );
    assert_eq!(
        serde_json::to_string(&Int32Value { value: 42 }).unwrap(),
        "42"
    );
    assert_eq!(
        serde_json::to_string(&Int64Value { value: 42 }).unwrap(),
        "\"42\"",
    );
    assert_eq!(
        serde_json::to_string(&UInt32Value { value: 42 }).unwrap(),
        "42"
    );
    assert_eq!(
        serde_json::to_string(&UInt64Value { value: 42 }).unwrap(),
        "\"42\"",
    );
    assert_eq!(
        serde_json::to_string(&FloatValue { value: 1.5 }).unwrap(),
        "1.5"
    );
    assert_eq!(
        serde_json::to_string(&DoubleValue { value: 1.5 }).unwrap(),
        "1.5",
    );
    assert_eq!(
        serde_json::to_string(&BytesValue {
            value: Bytes::from_static(b"hi"),
        })
        .unwrap(),
        "\"aGk=\"",
    );
}

#[test]
fn struct_value_and_list_value_use_native_json() {
    let value = Value {
        kind: Some(value::Kind::StructValue(Struct {
            fields: HashMap::from([
                ("flag".to_owned(), Value::from(true)),
                ("name".to_owned(), Value::from("scarlet")),
                (
                    "numbers".to_owned(),
                    Value::from([Value::from(1.0), Value::from(2.0)]),
                ),
            ]),
        })),
    };

    assert_eq!(
        serde_json::to_value(&value).unwrap(),
        serde_json::json!({
            "flag": true,
            "name": "scarlet",
            "numbers": [1.0, 2.0],
        }),
    );

    assert_eq!(
        serde_json::to_value(ListValue {
            values: vec![Value::from(true), Value::from("ok")],
        })
        .unwrap(),
        serde_json::json!([true, "ok"]),
    );
}

#[test]
fn field_mask_uses_canonical_comma_joined_camel_case_string() {
    let mask = FieldMask {
        paths: vec!["event_time".to_owned(), "actor.id".to_owned()],
    };

    assert_eq!(
        serde_json::to_string(&mask).unwrap(),
        "\"eventTime,actor.id\"",
    );

    let decoded: FieldMask = serde_json::from_str("\"eventTime,actor.id\"").unwrap();
    assert_eq!(decoded.paths, ["event_time", "actor.id"]);
}

#[test]
fn any_uses_protojson_type_url_field_name() {
    let any = Any {
        type_url: "type.googleapis.com/example.Event".to_owned(),
        value: Bytes::from_static(b"abc"),
    };

    let encoded = serde_json::to_value(&any).unwrap();

    assert!(encoded.get("@type").is_some(), "{encoded}");

    let decoded: Any =
        serde_json::from_value(serde_json::json!({"@type": any.type_url, "value": "YWJj"}))
            .unwrap();
    assert_eq!(decoded.value, Bytes::from_static(b"abc"));
}

#[test]
fn empty_uses_empty_json_object() {
    assert_eq!(
        serde_json::to_value(Empty {}).unwrap(),
        serde_json::json!({}),
    );
}
