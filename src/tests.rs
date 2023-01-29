use serde_json::json;
use crate::handler;

#[tokio::test]
async fn happy_path() {
    let json_str = r#"{
        "client_id":"one",
        "client_secret":"two",
        "audience": "three",
        "tenant": "four",
        "grant_type": "client_credentials"}"#;
    let input = serde_json::from_str(json_str).expect("failed to parse event");
    let context = lambda_runtime::Context::default();
    let event = lambda_runtime::LambdaEvent::new(input, context);

    let resp = match handler(event).await {
        Ok(r) => r,
        Err(e) => panic!("Error in handler: {:?}", e),
    };

    let expected_response = json!({
    "message": "token: eyJhbGciOiJIUzI1NiJ9.eyJhdWRpZW5jZSI6InRocmVlIiwiY2xpZW50X2lkIjoib25lIiwiY2xpZW50X3NlY3JldCI6InR3byIsImdyYW50X3R5cGUiOiJjbGllbnRfY3JlZGVudGlhbHMiLCJ0ZW5hbnQiOiJmb3VyIn0.jr1GdFV-n6DmMOEJLGJe3UbkgiO9pHcFBHEI658Q-Ig"
    });
    assert_eq!(resp, expected_response)
}

#[tokio::test]
#[should_panic(expected = "Claim value tenant is not ASCII")]
async fn panic_when_claim_contains_none_ascii_characters() {
    let json_str = r#"{
        "client_id":"one",
        "client_secret":"two",
        "audience": "three",
        "tenant": "INVALID❤",
        "grant_type": "client_credentials"}"#;
    let input = serde_json::from_str(json_str).expect("failed to parse event");
    let context = lambda_runtime::Context::default();
    let event = lambda_runtime::LambdaEvent::new(input, context);

    let resp = match handler(event).await {
        Ok(r) => r,
        Err(e) => panic!("Error in handler: {:?}", e),
    };

    let expected_response = json!({
    "message": "token: eyJhbGciOiJIUzI1NiJ9.eyJhdWRpZW5jZSI6InRocmVlIiwiY2xpZW50X2lkIjoib25lIiwiY2xpZW50X3NlY3JldCI6InR3byIsImdyYW50X3R5cGUiOiJjbGllbnRfY3JlZGVudGlhbHMiLCJ0ZW5hbnQiOiJmb3VyIn0.jr1GdFV-n6DmMOEJLGJe3UbkgiO9pHcFBHEI658Q-Ig"
    });
    assert_eq!(resp, expected_response)
}