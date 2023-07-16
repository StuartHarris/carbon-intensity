use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use self::{intensity::Period, location::Location};

pub mod factors;
pub mod global;
pub mod intensity;
pub mod location;
pub mod national_intensity;
pub mod national_mix;
pub mod postcode;
pub mod regional;

#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, PartialEq)]
pub enum Mode {
    #[default]
    National,
    Local,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub enum Scope {
    #[default]
    None,
    National(National),
    Local(Local),
}

pub trait DataSet {}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct National {
    pub generation_mix: Vec<Period>,
}

impl DataSet for National {}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Local {
    pub location: Option<Location>,
}
impl DataSet for Local {}

#[derive(Default, Serialize)]
pub struct Data<T: DataSet + Serialize> {
    pub scope: T,
    pub periods: Vec<Period>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Default, Serialize)]
pub struct Model {
    pub time: DateTime<Utc>,
    pub mode: Mode,
    pub national: Data<National>,
    pub local: Data<Local>,
}
