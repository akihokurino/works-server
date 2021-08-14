pub mod ddb;
pub mod domain;
pub mod firebase;
pub mod graphql;
pub mod misoca;
pub mod task;
pub mod util;

#[macro_use]
extern crate diesel;

use convert_case::{Case, Casing};
use jsonwebtoken;
use juniper::{graphql_value, FieldError};
use reqwest;
use strum_macros::Display as StrumDisplay;
use thiserror::Error as ThisErr;

const INVOICE_BUCKET: &str = "works-userdata";
const INVOICE_PDF_DOWNLOAD_DURATION: u32 = 86400;

#[derive(ThisErr, Debug, PartialOrd, PartialEq, Clone)]
pub enum CoreError {
    #[error("不正なパラメーターです: {0}")]
    BadRequest(String),
    #[error("認証エラーです")]
    UnAuthenticate,
    #[error("禁止された行為です")]
    Forbidden,
    #[error("指定されたリソースが見つかりません")]
    NotFound,
    #[error("サーバーエラーです: {0}")]
    Internal(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

impl From<String> for CoreError {
    fn from(v: String) -> Self {
        Self::Internal(v)
    }
}

impl From<diesel::result::Error> for CoreError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Self::NotFound,
            _ => Self::Internal(e.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for CoreError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(e: reqwest::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

#[derive(StrumDisplay, Debug)]
pub enum FieldErrorCode {
    BadRequest,
    UnAuthenticate,
    NotFound,
    Forbidden,
    Internal,
}

pub struct FieldErrorWithCode {
    err: CoreError,
    code: FieldErrorCode,
}

impl From<CoreError> for FieldErrorWithCode {
    fn from(err: CoreError) -> Self {
        FieldErrorWithCode {
            err: err.clone(),
            code: match err {
                CoreError::BadRequest(_) => FieldErrorCode::BadRequest,
                CoreError::UnAuthenticate => FieldErrorCode::UnAuthenticate,
                CoreError::Forbidden => FieldErrorCode::Forbidden,
                CoreError::NotFound => FieldErrorCode::NotFound,
                CoreError::Internal(_) => FieldErrorCode::Internal,
            },
        }
    }
}

impl From<cloud_storage::Error> for FieldErrorWithCode {
    fn from(_err: cloud_storage::Error) -> Self {
        FieldErrorWithCode {
            err: CoreError::Internal("GCSでエラーが発生しました".to_string()),
            code: FieldErrorCode::Internal,
        }
    }
}

impl From<FieldErrorWithCode> for FieldError {
    fn from(v: FieldErrorWithCode) -> Self {
        let code = v.code.to_string().to_case(Case::UpperSnake);

        FieldError::new(
            v.err,
            graphql_value!({
                "code": code,
            }),
        )
    }
}
