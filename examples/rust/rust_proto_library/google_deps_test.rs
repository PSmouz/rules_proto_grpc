use proto_runtime::prost::Message;
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
    };

    let mut bytes = Vec::new();
    session.encode(&mut bytes).unwrap();
    let decoded = Session::decode(bytes.as_slice()).unwrap();

    assert_eq!(decoded.name, "sessions/123");
    assert!(decoded.location.is_some());
    assert!(decoded.created_at.is_some());
}
