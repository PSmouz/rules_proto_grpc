use proto_runtime::prost::Message;
use proto_runtime::serde_json;
use identity_rust_proto::example::events::identity::v1::IdentityEvent;
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
    };

    let mut bytes = Vec::new();
    session.encode(&mut bytes).unwrap();
    let decoded = Session::decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.name, "sessions/123");
    assert!(decoded.location.is_some());
    assert!(decoded.created_at.is_some());

    let json = serde_json::to_string(&decoded).unwrap();
    assert!(json.contains(r#""behavior":"REQUIRED""#));
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
