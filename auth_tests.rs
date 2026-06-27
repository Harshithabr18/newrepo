//! Integration/unit tests for Signature V4.

use my_object_store::auth::signature_v4::{parse_auth_header, derive_signing_key};

#[test]
fn test_parse_auth_header() {
    let header = "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20130524/us-east-1/s3/aws4_request, SignedHeaders=host;range;x-amz-date, Signature=fef7f28e89430c4554ca4e6d12685790d7def37951a31e8b95dbd28a569b0b96";
    let parsed = parse_auth_header(header).unwrap();
    assert_eq!(parsed.access_key, "AKIAIOSFODNN7EXAMPLE");
    assert_eq!(parsed.date_stamp, "20130524");
    assert_eq!(parsed.region, "us-east-1");
    assert_eq!(parsed.service, "s3");
    assert_eq!(parsed.signed_headers, vec!["host", "range", "x-amz-date"]);
    assert_eq!(parsed.signature, "fef7f28e89430c4554ca4e6d12685790d7def37951a31e8b95dbd28a569b0b96");
}

#[test]
fn test_derive_signing_key() {
    let secret = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
    let date = "20130524";
    let region = "us-east-1";
    let service = "s3";

    let key = derive_signing_key(secret, date, region, service);
    let hex_key = hex::encode(key);
    assert_eq!(hex_key, "f117494eff5d09da21cbf7f0339559ea04fc9582d31299cb992be70a6b27c97a");
}
