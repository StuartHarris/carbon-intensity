use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize};

pub const INTENSITY_API: &str = "https://api.carbonintensity.org.uk";

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Period {
    #[serde(deserialize_with = "period_date_time_deserialize")]
    pub from: DateTime<Utc>,
    #[serde(deserialize_with = "period_date_time_deserialize")]
    pub to: DateTime<Utc>,
    pub intensity: Option<Intensity>,
    #[serde(rename = "generationmix")]
    pub generation_mix: Option<Vec<GenerationMix>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Intensity {
    pub forecast: i32,
    pub actual: Option<i32>,
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
