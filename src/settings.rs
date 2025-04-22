use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub light_levels: LightLevels,
    pub times: Times,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            light_levels: LightLevels::default(),
            times: Times::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightLevels {
    pub close: f64,
    pub open: f64,
}

impl Default for LightLevels {
    fn default() -> Self {
        Self {
            open: 100.0,
            close: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Times {
    pub open: chrono::NaiveTime,
    pub close: chrono::NaiveTime,
}

impl Default for Times {
    fn default() -> Self {
        Self {
            open: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            close: chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        }
    }
}

