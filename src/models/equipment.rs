use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransformerSpec {
    pub kva: f64,
    pub voltage_pri_kv: f64,
    pub voltage_sec_v: f64,
    pub rated_current_a: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakerSpec {
    pub name: String,
    pub frame_af: u32,
    pub trip_at: u32,
    pub poles: u8,
    pub icu_ka: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CableSpec {
    pub description: String,
    pub conductor_size_sqmm: f64,
    pub ground_size_sqmm: f64,
    pub conduit_size_mm: u32,
}
