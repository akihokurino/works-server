pub mod auth;

use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Debug, Deserialize)]
pub struct FirebaseConfig {
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
}

impl FirebaseConfig {
    pub fn new() -> FirebaseConfig {
        let cred_path =
            env::var("FIREBASE_CREDENTIALS").expect("should set firebase credential path");
        let file = File::open(cred_path).expect("cannot read firebase credential");
        let reader = BufReader::new(file);
        let config: FirebaseConfig = serde_json::from_reader(reader).unwrap();
        config
    }
}
