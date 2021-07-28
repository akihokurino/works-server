use crate::firebase::FirebaseConfig;
use crate::{CoreError, CoreResult};
use jsonwebtoken::{decode_header, Algorithm, DecodingKey, Validation};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
    pub sub: String,
    pub uid: Option<String>,
}

pub async fn verify_id_token(token: &str) -> CoreResult<String> {
    let firebase_config = FirebaseConfig::new();
    verify(token, &firebase_config)
        .await
        .map(|data| data.claims.sub)
}

async fn verify(
    token: &str,
    firebase_config: &FirebaseConfig,
) -> CoreResult<jsonwebtoken::TokenData<Claims>> {
    let project_id = firebase_config.project_id.clone();

    let kid = match decode_header(token).map(|header| header.kid) {
        Ok(Some(k)) => k,
        Ok(None) => {
            let err =
                jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::__Nonexhaustive);
            return Err(CoreError::from(err));
        }
        Err(err) => return Err(CoreError::from(err)),
    };

    let jwks = get_firebase_jwks().await.map_err(CoreError::from)?;
    let jwk = jwks.get(&kid).unwrap();

    let mut validation = Validation {
        iss: Some(format!(
            "https://securetoken.google.com/{}",
            project_id.clone()
        )),
        ..Validation::new(Algorithm::RS256)
    };

    validation.set_audience(&[project_id]);

    let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e);
    jsonwebtoken::decode::<Claims>(token, &key, &validation).map_err(CoreError::from)
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct JWK {
    pub e: String,
    pub alg: String,
    pub kty: String,
    pub kid: String,
    pub n: String,
}

#[derive(Debug, Deserialize)]
struct KeysResponse {
    keys: Vec<JWK>,
}

async fn get_firebase_jwks() -> Result<HashMap<String, JWK>, reqwest::Error> {
    let url =
        "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
    let resp = reqwest::get(url).await?.json::<KeysResponse>().await?;

    let mut key_map = HashMap::new();
    for key in resp.keys {
        key_map.insert(key.kid.clone(), key);
    }

    Ok(key_map)
}
