use serde::{Deserialize, Serialize};

use super::intensity::Period;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub data: Vec<Period>,
}
