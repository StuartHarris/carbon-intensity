use serde::{Deserialize, Serialize};

use crate::model::intensity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    pub from: String,
    pub to: String,
    pub intensity: Intensity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intensity {
    pub forecast: isize,
    pub actual: Option<isize>,
    pub index: String,
}

impl From<intensity::Period> for Period {
    fn from(value: intensity::Period) -> Self {
        Period {
            from: value.from.to_rfc3339(),
            to: value.to.to_rfc3339(),
            intensity: value.intensity.into(),
        }
    }
}

impl From<intensity::Intensity> for Intensity {
    fn from(value: intensity::Intensity) -> Self {
        Intensity {
            forecast: value.forecast,
            actual: value.actual,
            index: value.index,
        }
    }
}

impl From<intensity::Set> for Vec<Period> {
    fn from(value: intensity::Set) -> Self {
        value.all().into_iter().map(|p| p.into()).collect()
    }
}
