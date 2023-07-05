use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PostCodeResponse {
    pub status: String,
    pub result: Vec<PostCode>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PostCode {
    pub outcode: String,
    pub admin_district: String,
}
