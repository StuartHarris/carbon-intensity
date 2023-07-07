use serde::{Deserialize, Serialize};

use crate::{model::intensity, Scope};

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub scope: Scope,
    pub national_name: String,
    pub national: Vec<DataPoint>,
    pub local_name: String,
    pub local: Vec<DataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub date: String,
    pub forecast: i32,
    pub actual: Option<i32>,
    pub category: Category,
}

impl From<intensity::Period> for DataPoint {
    fn from(value: intensity::Period) -> Self {
        DataPoint {
            date: value.from.to_rfc3339(),
            forecast: value.intensity.forecast,
            actual: value.intensity.actual,
            category: Category::Total,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Total,
    Gas,
    Coal,
    Biomass,
    Nuclear,
    Hydro,
    Imports,
    Other,
    Wind,
    Solar,
}
