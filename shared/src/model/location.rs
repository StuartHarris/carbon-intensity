use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}
