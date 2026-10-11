use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseConnection {
    PhaseA,
    PhaseB,
    PhaseC,
    ThreePhase,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuxLoad {
    pub name: String,
    pub power_va: f64,
    pub phase: PhaseConnection,
}

impl AuxLoad {
    pub fn new(name: impl Into<String>, power_va: f64, phase: PhaseConnection) -> Self {
        Self {
            name: name.into(),
            power_va,
            phase,
        }
    }
}
