use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use super::intensity::{Period, INTENSITY_API};

pub(crate) fn url(from: &DateTime<Utc>) -> Url {
    let from = from.format("%Y-%m-%dT%H:%M").to_string() + "Z";
    let base = Url::parse(INTENSITY_API).unwrap();
    let url = base.join(&format!("/intensity/{from}/fw24h")).unwrap();
    url
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NationalResponse {
    pub data: Vec<Period>,
}
