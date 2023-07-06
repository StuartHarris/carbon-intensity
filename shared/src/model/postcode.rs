use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use super::location::Location;

pub const BASE_URL: &str = "https://api.postcodes.io";

pub fn url() -> String {
    let base = Url::parse(BASE_URL).unwrap();
    let url = base.join("/postcodes").unwrap();
    url.to_string()
}

#[derive(Serialize)]
pub struct Query {
    pub lat: f64,
    pub lon: f64,
}

impl From<Location> for Query {
    fn from(location: Location) -> Self {
        Self {
            lat: location.latitude,
            lon: location.longitude,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostcodeResponse {
    pub status: i64,
    pub result: Vec<Postcode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Postcode {
    pub postcode: String,
    pub quality: i64,
    pub eastings: i64,
    pub northings: i64,
    pub country: String,
    pub nhs_ha: String,
    pub longitude: f64,
    pub latitude: f64,
    pub european_electoral_region: String,
    pub primary_care_trust: String,
    pub region: String,
    pub lsoa: String,
    pub msoa: String,
    pub incode: String,
    pub outcode: String,
    pub parliamentary_constituency: String,
    pub admin_district: String,
    pub parish: String,
    pub admin_county: Value,
    pub date_of_introduction: String,
    pub admin_ward: String,
    pub ced: Value,
    pub ccg: String,
    pub nuts: String,
    pub pfa: String,
    pub codes: Codes,
    pub distance: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Codes {
    pub admin_district: String,
    pub admin_county: String,
    pub admin_ward: String,
    pub parish: String,
    pub parliamentary_constituency: String,
    pub ccg: String,
    pub ccg_id: String,
    pub ced: String,
    pub nuts: String,
    pub lsoa: String,
    pub msoa: String,
    pub lau2: String,
    pub pfa: String,
}
