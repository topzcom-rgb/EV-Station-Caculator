use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChargerType {
    DcFast(u32),
    AcType2(u32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChargerUnit {
    pub id: String,
    pub name: String,
    pub charger_type: ChargerType,
    pub power_kw: f64,
    pub distance_m: f64,
    pub power_factor: f64,
    pub efficiency: f64,
}

impl ChargerUnit {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        charger_type: ChargerType,
        power_kw: f64,
        distance_m: f64,
    ) -> Self {
        let (default_pf, default_eff) = match &charger_type {
            ChargerType::DcFast(_) => (0.98, 0.95),
            ChargerType::AcType2(_) => (0.98, 0.95),
        };

        Self {
            id: id.into(),
            name: name.into(),
            charger_type,
            power_kw,
            distance_m,
            power_factor: default_pf,
            efficiency: default_eff,
        }
    }
}
