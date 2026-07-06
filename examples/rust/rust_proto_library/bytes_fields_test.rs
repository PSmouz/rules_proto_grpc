use bytes_payload_rust_proto::example::bytes::BytesPayload;
use proto_runtime::prost::bytes::Bytes;

#[test]
fn ordinary_proto_bytes_fields_use_bytes() {
    let payload = BytesPayload {
        data: Bytes::from_static(b"payload"),
    };

    assert_eq!(payload.data, Bytes::from_static(b"payload"));
}
