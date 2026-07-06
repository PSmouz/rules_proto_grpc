use identity_rust_proto::example::events::identity::v1::IdentityEvent;
use keyword_consumer_rust_proto::example::keyword::consumer::KeywordConsumer;
use proto_runtime::prost::bytes::Bytes;
use proto_runtime::prost::Message;
use proto_runtime::serde_json;
use session_rust_proto::example::session::Session;

#[test]
fn googleapis_and_protobuf_deps_compile_through_single_deps_attr() {
    let session = Session {
        name: "sessions/123".to_string(),
        location: Some(google_type::google::r#type::LatLng {
            latitude: 52.52,
            longitude: 13.405,
        }),
        created_at: Some(google_protobuf::google::protobuf::Timestamp {
            seconds: 1_783_003_200,
            nanos: 0,
        }),
        behavior: google_api::google::api::FieldBehavior::Required as i32,
        mask: Some(google_protobuf::google::protobuf::FieldMask {
            paths: vec!["event_time".to_owned(), "actor.id".to_owned()],
        }),
        payload: Some(google_protobuf::google::protobuf::Any {
            type_url: "type.googleapis.com/example.Event".to_owned(),
            value: Bytes::from_static(b"abc"),
        }),
    };

    let mut bytes = Vec::new();
    session.encode(&mut bytes).unwrap();
    let decoded = Session::decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.name, "sessions/123");
    assert!(decoded.location.is_some());
    assert!(decoded.created_at.is_some());

    let json = serde_json::to_string(&decoded).unwrap();
    assert!(json.contains(r#""behavior":"REQUIRED""#));
    assert!(json.contains(r#""createdAt":"2026-07-02T14:40:00+00:00""#));
    assert!(json.contains(r#""mask":"eventTime,actor.id""#));
    assert!(json.contains(r#""@type":"type.googleapis.com/example.Event""#));
}

#[test]
fn nested_package_dependency_does_not_extern_current_crate_types() {
    let event = event_rust_proto::example::events::CloudEvent {
        id: "event-1".to_owned(),
        source: "test".to_owned(),
    };

    let identity_event = IdentityEvent {
        subject: "users/1".to_owned(),
        event: Some(event),
    };

    let mut bytes = Vec::new();
    identity_event.encode(&mut bytes).unwrap();
    let decoded = IdentityEvent::decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.subject, "users/1");
    assert!(decoded.event.is_some());
}

#[test]
fn extern_paths_escape_rust_keyword_segments() {
    let consumer = KeywordConsumer {
        role: Some(
            keyword_type_rust_proto::example::keyword::r#type::KeywordRole {
                name: "signer".to_owned(),
            },
        ),
    };

    let mut bytes = Vec::new();
    consumer.encode(&mut bytes).unwrap();
    let decoded = KeywordConsumer::decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.role.unwrap().name, "signer");
}
