use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub data: Vec<Region>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub region_id: i64,
    pub dno_region: String,
    pub short_name: String,
    pub postcode: String,
    pub data: Vec<Period>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Period {
    pub from: String,
    pub to: String,
    pub intensity: Intensity,
    pub generation_mix: Vec<GenerationMix>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Intensity {
    pub forecast: i64,
    pub index: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationMix {
    pub fuel: String,
    #[serde(rename = "perc")]
    pub percentage: f64,
}
