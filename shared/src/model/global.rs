use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug)]
pub struct Set {
    pub past: Vec<Period>,
    pub future: Vec<Period>,
}

impl Set {
    pub fn all(&self) -> Vec<Period> {
        let mut all = self.past.clone();
        all.extend(self.future.clone());
        all
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub data: Vec<Period>,
}

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
