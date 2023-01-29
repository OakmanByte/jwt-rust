mod tests;

extern crate serde;
extern crate serde_json;
extern crate lambda_runtime;

use std::collections::BTreeMap;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;
use lambda_http::{service_fn, Error};
use lambda_runtime::LambdaEvent;
use rusoto_core::{Region, RusotoError};
use rusoto_secretsmanager::{SecretsManager, SecretsManagerClient, GetSecretValueRequest, GetSecretValueError};
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

async fn get_secret(name: &str) -> Result<String, RusotoError<GetSecretValueError>> {
    let client = SecretsManagerClient::new(Region::default());
    let request = GetSecretValueRequest {
        secret_id: name.to_string(),
        ..Default::default()
    };
    let result = client.get_secret_value(request).await?;
    return Ok(result.secret_string.expect("Could not parse the received secret value"))
}

fn validate_claims(claims: &BTreeMap<&str, String>) -> Result<(), Error> {
    for (claim_name, claim_value) in claims {
        if !claim_value.is_ascii() {
            return Err(lambda_runtime::Error::from(format!("Claim value {} is not ASCII", claim_name)));
        }
    }
    Ok(())
}

async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let body: Value = event.payload;
    let client_request: ClientRequest = match serde_json::from_value(body) {
        Ok(client_request) => client_request,
        Err(error) => {
            return Err(Error::from(error));
        }
    };

    let private_key: Hmac<Sha256> = Hmac::new_from_slice(b"some-secret")?;
    let mut claims = BTreeMap::new();

    claims.insert("client_id", client_request.client_id);
    claims.insert("client_secret", client_request.client_secret);
    claims.insert("audience", client_request.audience);
    claims.insert("tenant", client_request.tenant);
    claims.insert("grant_type", client_request.grant_type);

    validate_claims(&claims)?;

    let token_str = claims.sign_with_key(&private_key)?;
    println!("token:{}", token_str);

    Ok(json!({ "message": format!("token: {}",token_str) }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}