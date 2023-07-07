use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Location {
    pub coordinate: Coordinate,
    pub outcode: String,
    pub admin_district: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}
