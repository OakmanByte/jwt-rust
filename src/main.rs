extern crate serde;
extern crate serde_json;
extern crate lambda_runtime;

use std::collections::BTreeMap;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;
use lambda_http::{service_fn, Error};
use lambda_runtime::LambdaEvent;

use serde::{Deserialize};
use serde_json::{json, Value};


#[derive(Debug, Deserialize)]
struct ClientRequest {
    client_id: String,
    client_secret: String,
    audience: String,
    tenant: String,
    grant_type: String,
}

/// This is the main body for the function.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // Extract some useful information from the request
    let body: Value = event.payload;
    let client_request: ClientRequest = match serde_json::from_value(body) {
        Ok(client_request) => client_request,
        Err(error) => {
            return Err(Error::from(error));
        }
    };

    let key: Hmac<Sha256> = Hmac::new_from_slice(b"some-secret")?;
    let mut claims = BTreeMap::new();
    claims.insert("client_id", client_request.client_id);
    claims.insert("client_secret", client_request.client_secret);
    claims.insert("audience", client_request.audience);
    claims.insert("tenant", client_request.tenant);
    claims.insert("grant_type", client_request.grant_type);

    //Validation
    let token_str = claims.sign_with_key(&key)?;
    println!("token:{}", token_str);

    Ok(json!({ "message": format!("token: {}",token_str) }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

#[tokio::test]
async fn test() {
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