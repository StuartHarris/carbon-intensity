use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::model::global;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub intensity: Intensity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intensity {
    pub forecast: isize,
    pub actual: Option<isize>,
    pub index: String,
}

impl From<global::Period> for Period {
    fn from(value: global::Period) -> Self {
        Period {
            from: value.from,
            to: value.to,
            intensity: value.intensity.into(),
        }
    }
}

impl From<global::Intensity> for Intensity {
    fn from(value: global::Intensity) -> Self {
        Intensity {
            forecast: value.forecast,
            actual: value.actual,
            index: value.index,
        }
    }
}

impl From<global::Set> for Vec<Period> {
    fn from(value: global::Set) -> Self {
        value.all().into_iter().map(|p| p.into()).collect()
    }
}
