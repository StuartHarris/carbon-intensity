use serde::{Deserialize, Serialize};

use self::intensity::Set;

pub mod global;
pub mod intensity;
pub mod location;
pub mod postcode;
pub mod regional;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum Mode {
    #[default]
    National,
    Here,
}

#[derive(Default)]
pub struct Model {
    pub mode: Mode,
    pub outcode: Option<String>,
    pub admin_district: Option<String>,
    pub national: Set,
    pub here: Set,
}
