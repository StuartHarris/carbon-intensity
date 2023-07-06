use serde::{Deserialize, Serialize};

use self::intensity::Period;

pub mod factors;
pub mod global;
pub mod intensity;
pub mod location;
pub mod postcode;
pub mod regional;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    National,
    Local,
}

#[derive(Default)]
pub struct Model {
    pub mode: Mode,
    pub outcode: Option<String>,
    pub admin_district: Option<String>,
    pub periods: Vec<Period>,
}
