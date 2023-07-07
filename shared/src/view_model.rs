use serde::{Deserialize, Serialize};

use crate::model::{intensity::Period, Model};

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
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
    pub mix: Mix,
}

impl From<&Model> for ViewModel {
    fn from(value: &Model) -> Self {
        let location = value.local.scope.location.clone();
        ViewModel {
            national_name: "UK".to_string(),
            national: value
                .national
                .periods
                .clone()
                .into_iter()
                .zip(value.national.scope.generation_mix.clone().into_iter())
                .map(DataPoint::from)
                .collect(),
            local_name: if location.is_some() {
                format!(
                    "{area}, {code}",
                    area = location.clone().unwrap().admin_district,
                    code = location.unwrap().outcode,
                )
            } else {
                "Local".to_string()
            },
            local: value
                .local
                .periods
                .clone()
                .into_iter()
                .map(|p| DataPoint::from((p.clone(), p)))
                .collect(),
        }
    }
}

impl From<(Period, Period)> for DataPoint {
    fn from(value: (Period, Period)) -> Self {
        let mix = match value.1.generation_mix {
            Some(mixes) => mixes.iter().fold(Mix::default(), |mut mix, gen_mix| {
                match gen_mix.fuel.as_ref() {
                    "gas" => {
                        mix.gas = gen_mix.percentage;
                        mix
                    }
                    "coal" => {
                        mix.coal = gen_mix.percentage;
                        mix
                    }
                    "biomass" => {
                        mix.biomass = gen_mix.percentage;
                        mix
                    }
                    "nuclear" => {
                        mix.nuclear = gen_mix.percentage;
                        mix
                    }
                    "hydro" => {
                        mix.hydro = gen_mix.percentage;
                        mix
                    }
                    "imports" => {
                        mix.imports = gen_mix.percentage;
                        mix
                    }
                    "other" => {
                        mix.other = gen_mix.percentage;
                        mix
                    }
                    "wind" => {
                        mix.wind = gen_mix.percentage;
                        mix
                    }
                    "solar" => {
                        mix.solar = gen_mix.percentage;
                        mix
                    }
                    _ => mix,
                }
            }),
            None => Default::default(),
        };
        DataPoint {
            date: value.0.from.to_rfc3339(),
            forecast: value
                .0
                .intensity
                .clone()
                .map(|f| f.forecast)
                .unwrap_or_default(),
            actual: value.0.intensity.map(|f| f.actual).unwrap_or_default(),
            mix,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mix {
    gas: f32,
    coal: f32,
    biomass: f32,
    nuclear: f32,
    hydro: f32,
    imports: f32,
    other: f32,
    wind: f32,
    solar: f32,
}
