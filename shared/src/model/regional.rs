use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use super::intensity::{Period, INTENSITY_API};

pub(crate) fn url(from: &DateTime<Utc>, outcode: &str) -> Url {
    let from = from.format("%Y-%m-%dT%H:%M").to_string() + "Z";
    let base = Url::parse(INTENSITY_API).unwrap();
    let url = base
        .join(&format!(
            "/regional/intensity/{from}/fw24h/postcode/{outcode}"
        ))
        .unwrap();
    url
}

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
