use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use self::intensity::Period;

pub mod factors;
pub mod global;
pub mod intensity;
pub mod location;
pub mod national;
pub mod postcode;
pub mod regional;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum Scope {
    #[default]
    National,
    Local,
}

#[derive(Default)]
pub struct Model {
    pub scope: Scope,
    pub time: DateTime<Utc>,
    pub outcode: Option<String>,
    pub admin_district: Option<String>,
    pub national: Vec<Period>,
    pub national_updated: DateTime<Utc>,
    pub local: Vec<Period>,
    pub local_updated: DateTime<Utc>,
}
