use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{
    intensity::{GenerationMix, Period},
    Mode, Model,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub mode: Mode,
    pub national_name: String,
    pub national_intensity: Vec<IntensityPoint>,
    pub national_mix: Vec<GenerationMixPoint>,
    pub local_name: String,
    pub local_intensity: Vec<IntensityPoint>,
    pub local_mix: Vec<GenerationMixPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntensityPoint {
    pub date: String,
    pub forecast: i32,
    pub actual: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMixPoint {
    pub date: String,
    pub fuel: String,
    #[serde(rename = "perc")]
    pub percentage: f32,
}

impl From<&Model> for ViewModel {
    fn from(value: &Model) -> Self {
        let mix_order: HashMap<String, u8> = [
            "Coal", "Gas", "Imports", "Biomass", "Nuclear", "Hydro", "Wind", "Solar",
        ]
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, s)| (s.to_string(), i as u8))
        .collect();

        let mut national_mix: Vec<GenerationMixPoint> = value
            .national
            .scope
            .generation_mix
            .clone()
            .into_iter()
            .flat_map(|period| {
                period
                    .generation_mix
                    .clone()
                    .into_iter()
                    .flat_map(move |mix| {
                        let period = period.clone();
                        mix.clone()
                            .into_iter()
                            .map(move |mix| GenerationMixPoint::from((period.clone(), mix)))
                    })
            })
            .filter(|m| m.fuel != "Other")
            .collect();
        national_mix.sort_by(|a, b| mix_order[&a.fuel].cmp(&mix_order[&b.fuel]));

        let national_intensity = value
            .national
            .periods
            .clone()
            .into_iter()
            .map(IntensityPoint::from)
            .collect();

        let location = value.local.scope.location.clone();
        let local_name = if location.is_some() {
            format!(
                "{area}, {code}",
                area = location.clone().unwrap().admin_district,
                code = location.unwrap().outcode,
            )
        } else {
            "Local".to_string()
        };

        let local_intensity = value
            .local
            .periods
            .clone()
            .into_iter()
            .map(|p| IntensityPoint::from(p))
            .collect();

        let mut local_mix: Vec<GenerationMixPoint> = value
            .local
            .periods
            .clone()
            .into_iter()
            .flat_map(|period| {
                period
                    .generation_mix
                    .clone()
                    .into_iter()
                    .flat_map(move |mix| {
                        let period = period.clone();
                        mix.clone()
                            .into_iter()
                            .map(move |mix| GenerationMixPoint::from((period.clone(), mix)))
                    })
            })
            .filter(|m| m.fuel != "Other")
            .collect();
        local_mix.sort_by(|a, b| mix_order[&a.fuel].cmp(&mix_order[&b.fuel]));

        ViewModel {
            mode: value.mode,
            national_name: "UK".to_string(),
            national_intensity,
            national_mix,
            local_name,
            local_intensity,
            local_mix,
        }
    }
}

impl From<Period> for IntensityPoint {
    fn from(value: Period) -> Self {
        IntensityPoint {
            date: value.from.to_rfc3339(),
            forecast: value
                .intensity
                .clone()
                .map(|f| f.forecast)
                .unwrap_or_default(),
            actual: value.intensity.map(|f| f.actual).unwrap_or_default(),
        }
    }
}

impl From<(Period, GenerationMix)> for GenerationMixPoint {
    fn from(value: (Period, GenerationMix)) -> Self {
        GenerationMixPoint {
            date: value.0.from.to_rfc3339(),
            fuel: uppercase_first(&value.1.fuel),
            percentage: value.1.percentage,
        }
    }
}

fn uppercase_first(text: &str) -> String {
    assert!(text.len() >= 2, "text must be at least 2 chars long");
    text.chars()
        .next()
        .unwrap()
        .to_ascii_uppercase()
        .to_string()
        + &text[1..]
}
