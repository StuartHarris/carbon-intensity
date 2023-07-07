use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use self::{intensity::Period, location::Location};

pub mod factors;
pub mod global;
pub mod intensity;
pub mod location;
pub mod national;
pub mod national_mix;
pub mod postcode;
pub mod regional;

#[derive(Serialize, Default, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Default, Serialize, PartialEq, Eq, Debug, Clone, Copy)]
pub enum CurrentQuery {
    #[default]
    National,
    Local,
}

#[derive(Default, Serialize)]
pub struct Model {
    pub time: DateTime<Utc>,
    pub current_query: CurrentQuery, // note this is problematic and really should be some sort of context
    pub national: Data<National>,
    pub local: Data<Local>,
}
