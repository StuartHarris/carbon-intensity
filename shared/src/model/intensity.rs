use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Period {
    #[serde(deserialize_with = "period_date_time_deserialize")]
    pub from: DateTime<Utc>,
    #[serde(deserialize_with = "period_date_time_deserialize")]
    pub to: DateTime<Utc>,
    pub intensity: Intensity,
    #[serde(rename = "generationmix")]
    pub generation_mix: Option<Vec<GenerationMix>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Intensity {
    pub forecast: isize,
    pub actual: Option<isize>,
    pub index: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationMix {
    pub fuel: String,
    #[serde(rename = "perc")]
    pub percentage: f32,
}

pub fn period_date_time_deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT: &'static str = "%Y-%m-%dT%H:%M%Z";
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, FORMAT)
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_custom_date_time_deserialization() {
        let json_str = r#"
        {
          "from": "2023-07-04T23:00Z",
          "to": "2023-07-04T23:30Z",
          "intensity": {
            "forecast": 123,
            "actual": 456,
            "index": "moderate"
          }
        }
        "#;

        let data: Period = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            data.from,
            Utc.with_ymd_and_hms(2023, 7, 4, 23, 0, 0).unwrap()
        );
        assert_eq!(
            data.to,
            Utc.with_ymd_and_hms(2023, 7, 4, 23, 30, 0).unwrap()
        );
    }
}
