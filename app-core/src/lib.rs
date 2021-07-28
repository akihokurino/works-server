pub mod ddb;
pub mod domain;
pub mod errors;
pub mod firebase;
pub mod graphql;
pub mod misoca;
pub mod task;
pub mod util;

#[macro_use]
extern crate diesel;

const INVOICE_BUCKET: &str = "works-userdata";
const INVOICE_PDF_DOWNLOAD_DURATION: u32 = 86400;
