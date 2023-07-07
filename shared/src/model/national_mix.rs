use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use super::intensity::{Period, INTENSITY_API};

pub(crate) fn url(from: &DateTime<Utc>) -> Url {
    let to = *from + Duration::hours(24);

    let from = from.format("%Y-%m-%dT%H:%M").to_string() + "Z";
    let to = to.format("%Y-%m-%dT%H:%M").to_string() + "Z";
    let base = Url::parse(INTENSITY_API).unwrap();
    let url = base.join(&format!("/generation/{from}/{to}")).unwrap();
    url
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NationalMixResponse {
    pub data: Vec<Period>,
}
