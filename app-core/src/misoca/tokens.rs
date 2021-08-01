use crate::misoca::{CallInput, Client};
use crate::{CoreError, CoreResult};
use reqwest::Method;
use serde::{Deserialize, Serialize};

impl Client {
    pub async fn get_tokens(&self, input: get_tokens::Input) -> CoreResult<get_tokens::Output> {
        #[derive(Debug, Serialize)]
        struct Body {
            pub client_id: String,
            pub client_secret: String,
            pub redirect_uri: String,
            pub grant_type: String,
            pub code: String,
        }

        let body = Body {
            client_id: self.client_id.clone(),
            client_secret: self.secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
            grant_type: "authorization_code".to_string(),
            code: input.code.clone(),
        };

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![];

        self.call(
            CallInput {
                method: Method::POST,
                path: "/oauth2/token".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| CoreError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            "".to_string(),
        )
        .await?
        .error_for_status()?
        .json::<get_tokens::Output>()
        .await
        .map_err(CoreError::from)
    }

    pub async fn refresh_tokens(
        &self,
        input: refresh_tokens::Input,
    ) -> CoreResult<refresh_tokens::Output> {
        #[derive(Debug, Serialize)]
        struct Body {
            pub client_id: String,
            pub client_secret: String,
            pub redirect_uri: String,
            pub grant_type: String,
            pub refresh_token: String,
        }

        let body = Body {
            client_id: self.client_id.clone(),
            client_secret: self.secret.clone(),
            redirect_uri: self.redirect_uri.clone(),
            grant_type: "refresh_token".to_string(),
            refresh_token: input.refresh_token,
        };

        println!("json body: {}", serde_json::to_string(&body).unwrap());

        let query = vec![];

        self.call(
            CallInput {
                method: Method::POST,
                path: "/oauth2/token".to_string(),
                body: Some(
                    serde_json::to_string(&body)
                        .map_err(|e| CoreError::Internal(e.to_string()))?
                        .into(),
                ),
                query,
            },
            "".to_string(),
        )
        .await?
        .error_for_status()?
        .json::<refresh_tokens::Output>()
        .await
        .map_err(CoreError::from)
    }
}

pub mod get_tokens {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub code: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub access_token: String,
        pub refresh_token: String,
    }
}

pub mod refresh_tokens {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct Input {
        pub refresh_token: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Output {
        pub access_token: String,
        pub refresh_token: String,
    }
}
