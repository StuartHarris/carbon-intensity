use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub data: Vec<Factors>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Factors {
    #[serde(rename = "Biomass")]
    pub biomass: i64,
    #[serde(rename = "Coal")]
    pub coal: i64,
    #[serde(rename = "Dutch Imports")]
    pub dutch_imports: i64,
    #[serde(rename = "French Imports")]
    pub french_imports: i64,
    #[serde(rename = "Gas (Combined Cycle)")]
    pub gas_combined_cycle: i64,
    #[serde(rename = "Gas (Open Cycle)")]
    pub gas_open_cycle: i64,
    #[serde(rename = "Hydro")]
    pub hydro: i64,
    #[serde(rename = "Irish Imports")]
    pub irish_imports: i64,
    #[serde(rename = "Nuclear")]
    pub nuclear: i64,
    #[serde(rename = "Oil")]
    pub oil: i64,
    #[serde(rename = "Other")]
    pub other: i64,
    #[serde(rename = "Pumped Storage")]
    pub pumped_storage: i64,
    #[serde(rename = "Solar")]
    pub solar: i64,
    #[serde(rename = "Wind")]
    pub wind: i64,
}

#[allow(dead_code)]
impl Factors {
    pub fn new() -> Self {
        Self {
            biomass: 120,
            coal: 937,
            dutch_imports: 474,
            french_imports: 53,
            gas_combined_cycle: 394,
            gas_open_cycle: 651,
            hydro: 0,
            irish_imports: 458,
            nuclear: 0,
            oil: 935,
            other: 300,
            pumped_storage: 0,
            solar: 0,
            wind: 0,
        }
    }
}
