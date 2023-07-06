use serde::Deserialize;
use serde::Serialize;

use super::intensity::Period;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionalResponse {
    pub data: Region,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    #[serde(rename = "regionid")]
    pub region_id: i64,
    #[serde(rename = "dnoregion")]
    pub dno_region: String,
    pub shortname: String,
    pub postcode: String,
    pub data: Vec<Period>,
}
