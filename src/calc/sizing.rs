use crate::models::{BreakerSpec, CableSpec, TransformerSpec};
use serde::{Deserialize, Serialize};

/// พิกัดหม้อแปลงมาตรฐาน กฟภ. (22 kV / 400-230 V)
pub const STANDARD_PEA_TRANSFORMERS_KVA: &[f64] =
    &[160.0, 250.0, 315.0, 500.0, 630.0, 800.0, 1000.0];

/// ตารางพิกัดกระแสปลอดภัยและค่า Impedance ของสายทองแดงแกนเดี่ยวฉนวน XLPE (FD-0.6/1kV-CV) ร้อยท่อในอากาศ/ใต้ดิน
#[derive(Debug, Clone, Copy)]
pub struct CableAmpacityData {
    pub size_sqmm: f64,
    /// พิกัดกระแสสูงสุด (A) อ้างอิงตามกลุ่มการติดตั้งในท่อร้อยสาย
    pub ampacity_a: f64,
    /// ความต้านทานกระแสสลับ R (Ohm/km) ที่ 90°C
    pub r_ohm_per_km: f64,
    /// ค่ารีแอกแตนซ์ X (Ohm/km) ที่ 50 Hz
    pub x_ohm_per_km: f64,
    /// ขนาดท่อร้อยสายที่แนะนำ (mm OD เช่น HDPE)
    pub conduit_size_mm: u32,
}

pub const STANDARD_CV_CABLES: &[CableAmpacityData] = &[
    CableAmpacityData {
        size_sqmm: 16.0,
        ampacity_a: 76.0,
        r_ohm_per_km: 1.47,
        x_ohm_per_km: 0.086,
        conduit_size_mm: 32,
    },
    CableAmpacityData {
        size_sqmm: 25.0,
        ampacity_a: 101.0,
        r_ohm_per_km: 0.927,
        x_ohm_per_km: 0.083,
        conduit_size_mm: 40,
    },
    CableAmpacityData {
        size_sqmm: 35.0,
        ampacity_a: 125.0,
        r_ohm_per_km: 0.669,
        x_ohm_per_km: 0.081,
        conduit_size_mm: 50,
    },
    CableAmpacityData {
        size_sqmm: 50.0,
        ampacity_a: 151.0,
        r_ohm_per_km: 0.494,
        x_ohm_per_km: 0.079,
        conduit_size_mm: 50,
    },
    CableAmpacityData {
        size_sqmm: 70.0,
        ampacity_a: 192.0,
        r_ohm_per_km: 0.342,
        x_ohm_per_km: 0.076,
        conduit_size_mm: 65,
    },
    CableAmpacityData {
        size_sqmm: 95.0,
        ampacity_a: 232.0,
        r_ohm_per_km: 0.247,
        x_ohm_per_km: 0.075,
        conduit_size_mm: 80,
    },
    CableAmpacityData {
        size_sqmm: 120.0,
        ampacity_a: 269.0,
        r_ohm_per_km: 0.196,
        x_ohm_per_km: 0.073,
        conduit_size_mm: 90,
    },
    CableAmpacityData {
        size_sqmm: 150.0,
        ampacity_a: 300.0,
        r_ohm_per_km: 0.159,
        x_ohm_per_km: 0.073,
        conduit_size_mm: 100,
    },
    CableAmpacityData {
        size_sqmm: 185.0,
        ampacity_a: 341.0,
        r_ohm_per_km: 0.128,
        x_ohm_per_km: 0.072,
        conduit_size_mm: 110,
    },
    CableAmpacityData {
        size_sqmm: 240.0,
        ampacity_a: 400.0,
        r_ohm_per_km: 0.098,
        x_ohm_per_km: 0.071,
        conduit_size_mm: 125,
    },
    CableAmpacityData {
        size_sqmm: 300.0,
        ampacity_a: 458.0,
        r_ohm_per_km: 0.079,
        x_ohm_per_km: 0.070,
        conduit_size_mm: 140,
    },
    CableAmpacityData {
        size_sqmm: 400.0,
        ampacity_a: 546.0,
        r_ohm_per_km: 0.062,
        x_ohm_per_km: 0.069,
        conduit_size_mm: 160,
    },
];

/// ตารางพิกัดสายดินตามมาตรฐาน วสท. เทียบกับขนาดสายเฟส
pub fn select_ground_wire_size_sqmm(phase_size_sqmm: f64) -> (f64, u32) {
    // สายดิน 0.45/0.75kV-IEC01:
    // สายเฟส <= 16 -> สายดิน 10 (ท่อ 20)
    // สายเฟส <= 35 -> สายดิน 16 (ท่อ 25)
    // สายเฟส <= 70 -> สายดิน 35 (ท่อ 32)
    // สายเฟส <= 120 -> สายดิน 50 (ท่อ 32)
    // สายเฟส <= 185 -> สายดิน 95 (ท่อ 32) ตาม SLD
    // สายเฟส > 185 -> สายดิน 95 หรือ 120 (ท่อ 40)
    if phase_size_sqmm <= 16.0 {
        (10.0, 20)
    } else if phase_size_sqmm <= 35.0 {
        (16.0, 25)
    } else if phase_size_sqmm <= 70.0 {
        (35.0, 32)
    } else if phase_size_sqmm <= 120.0 {
        (50.0, 32)
    } else if phase_size_sqmm <= 185.0 {
        (95.0, 32)
    } else {
        (120.0, 40)
    }
}

/// ผลลัพธ์การคัดเลือกสายไฟฟ้าพร้อมการตรวจสอบ Voltage Drop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CableSizingResult {
    pub cable_spec: CableSpec,
    pub ampacity_a: f64,
    pub voltage_drop_v: f64,
    pub voltage_drop_percent: f64,
    pub length_m: f64,
    pub current_a: f64,
    pub upsized_due_to_vd: bool,
}

/// 1. Sizing หม้อแปลงไฟฟ้าตามขนาด kVA รวม
pub fn size_transformer(total_demand_kva: f64) -> TransformerSpec {
    let selected_kva = STANDARD_PEA_TRANSFORMERS_KVA
        .iter()
        .copied()
        .find(|&kva| kva >= total_demand_kva)
        .unwrap_or_else(|| {
            *STANDARD_PEA_TRANSFORMERS_KVA
                .last()
                .expect("transformer list is not empty")
        });

    let voltage_sec_v = 400.0;
    let rated_current_a = (selected_kva * 1000.0) / (3.0_f64.sqrt() * voltage_sec_v);

    TransformerSpec {
        kva: selected_kva,
        voltage_pri_kv: 22.0,
        voltage_sec_v,
        rated_current_a,
    }
}

/// มาตรฐานขนาด Frame (AF) และ Trip (AT) ของ Molded Case Circuit Breaker (MCCB)
const STANDARD_BREAKER_SIZES: &[(u32, u32)] = &[
    (16, 50),
    (20, 50),
    (32, 50),
    (40, 50),
    (50, 50),
    (63, 100),
    (80, 100),
    (100, 100),
    (125, 250),
    (160, 250),
    (200, 250),
    (225, 250),
    (250, 250),
    (300, 400),
    (320, 400),
    (350, 400),
    (400, 400),
    (500, 630),
    (630, 630),
    (800, 800),
    (1000, 1000),
    (1250, 1250),
];

/// 2. Breaker Selection: Main Breaker (MCCB)
/// คำนวณขนาด Trip (AT) เผื่อ Factor 1.25 เท่าของกระแสต่อเนื่อง และ Frame (AF) พร้อม Icu >= 50kA
/// (ตามมาตรฐานวิศวกรรมไฟฟ้า กรณีพิกัดมาตรฐานใกล้เคียง เช่น 268A เผื่อ 1.25 ได้ 335A หรือเลือกใช้พิกัดมาตรฐาน 320AT/400AF ตามแบบ SLD)
pub fn size_main_breaker(continuous_current_a: f64) -> BreakerSpec {
    let target_trip = continuous_current_a * 1.25;

    // หาพิกัด AT ที่ครอบคลุม target_trip หรือหาก target_trip อยู่ภายในพิกัด 320AT (+5% margin: 320 * 1.05 = 336A)
    let (trip_at, frame_af) = STANDARD_BREAKER_SIZES
        .iter()
        .find(|&&(at, _)| (at as f64) >= target_trip || (at == 320 && target_trip <= 336.0))
        .copied()
        .unwrap_or_else(|| {
            *STANDARD_BREAKER_SIZES
                .last()
                .expect("breaker list is not empty")
        });

    BreakerSpec {
        name: format!("Main MCCB {}AT/{}AF 3P 50kA", trip_at, frame_af),
        frame_af,
        trip_at,
        poles: 3,
        icu_ka: 50,
    }
}

/// Breaker Selection: Branch Breaker สำหรับตู้ EV Charger
pub fn size_ev_branch_breaker(rated_current_a: f64) -> BreakerSpec {
    let target_trip = rated_current_a * 1.25;
    let (trip_at, frame_af) = STANDARD_BREAKER_SIZES
        .iter()
        .find(|&&(at, _)| (at as f64) >= target_trip || (at == 320 && target_trip <= 336.0))
        .copied()
        .unwrap_or((320, 400));

    BreakerSpec {
        name: format!("EV Branch MCCB {}AT/{}AF 3P 50kA", trip_at, frame_af),
        frame_af,
        trip_at,
        poles: 3,
        icu_ka: 50,
    }
}

/// Breaker Selection: Branch Breaker สำหรับตู้ LP (Lighting & Power Panel)
pub fn size_lp_branch_breaker() -> BreakerSpec {
    BreakerSpec {
        name: "LP Branch Breaker 32AT/50AF 2P 18kA".to_string(),
        frame_af: 50,
        trip_at: 32,
        poles: 2,
        icu_ka: 18,
    }
}

/// 4. คำนวณ Voltage Drop (แรงดันตก 3 เฟส Line-to-Line)
///
/// สูตร:
/// $$\Delta V = \frac{\sqrt{3} \times I \times L \times (R \cos\theta + X \sin\theta)}{1000}$$
pub fn calculate_voltage_drop_3phase(
    current_a: f64,
    length_m: f64,
    r_ohm_per_km: f64,
    x_ohm_per_km: f64,
    power_factor: f64,
) -> f64 {
    let cos_theta = power_factor;
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let z_effective = r_ohm_per_km * cos_theta + x_ohm_per_km * sin_theta;

    (3.0_f64.sqrt() * current_a * length_m * z_effective) / 1000.0
}

/// ขนาดสายไฟสูงสุดที่แนะนำต่อ 1 วงจร (mm²) ตามมาตรฐานการจัดสายเข้าตู้สวิตช์บอร์ด/ตู้ชาร์จ
/// สายขนาดเกิน 240 mm² รัศมีดัดโค้งใหญ่มากและจัดสายเข้าเบรกเกอร์/บัสบาร์ในตู้ไม่ได้จริงหน้างาน
pub const MAX_RECOMMENDED_CABLE_SIZE_SQMM: f64 = 240.0;

/// 3 & 4. Cable & Conduit Selection พร้อม Voltage Drop Validation
/// - จำกัดขนาดสายเดี่ยวไม่เกิน 240 mm² ตามข้อกำหนดความสามารถในการจัดสายเข้าตู้
/// - หากกระแสหรือ Voltage Drop สูงเกินกว่าสาย 240 mm² เส้นเดียวจะรับได้ ระบบจะคำนวณแบ่งเป็นสายควบขนาน (Parallel Sets: เช่น 2x(4x1/C-185 mm²))
pub fn size_cable_with_vd(
    current_a: f64,
    length_m: f64,
    power_factor: f64,
    system_voltage_v: f64,
    max_vd_percent: f64,
) -> CableSizingResult {
    let min_ampacity_needed = current_a * 1.25;

    // รายการสายที่ยอมรับได้ (ไม่เกิน 240 mm²)
    let allowed_cables: Vec<&CableAmpacityData> = STANDARD_CV_CABLES
        .iter()
        .filter(|c| c.size_sqmm <= MAX_RECOMMENDED_CABLE_SIZE_SQMM)
        .collect();

    // ทดสอบจำนวนชุดสายควบ (sets = 1, 2, 3, 4, ...)
    for sets in 1..=6 {
        let _current_per_set = current_a / (sets as f64);
        let min_ampacity_per_set = min_ampacity_needed / (sets as f64);

        // หาขนาดสายที่รับกระแสพิกัดได้
        if let Some(start_idx) = allowed_cables
            .iter()
            .position(|c| c.ampacity_a >= min_ampacity_per_set)
        {
            let mut selected_idx = start_idx;
            let mut vd_v = 0.0;
            let mut vd_percent = 0.0;

            // ตรวจสอบ Voltage Drop โดยรวม (สายควบ R_eq = R/sets, X_eq = X/sets)
            while selected_idx < allowed_cables.len() {
                let cable = allowed_cables[selected_idx];
                let r_eff = cable.r_ohm_per_km / (sets as f64);
                let x_eff = cable.x_ohm_per_km / (sets as f64);

                vd_v = calculate_voltage_drop_3phase(
                    current_a,
                    length_m,
                    r_eff,
                    x_eff,
                    power_factor,
                );
                vd_percent = (vd_v / system_voltage_v) * 100.0;

                if vd_percent <= max_vd_percent || selected_idx == allowed_cables.len() - 1 {
                    break;
                }
                selected_idx += 1;
            }

            if vd_percent <= max_vd_percent || sets == 6 {
                let final_cable = allowed_cables[selected_idx];
                let (ground_sqmm, _ground_conduit_mm) =
                    select_ground_wire_size_sqmm(final_cable.size_sqmm);

                let description = if sets == 1 {
                    format!(
                        "4x1/C-{} mm² FD-0.6/1kV-CV in HDPE {} mm + 1-{} mm² IEC01 (Ground)",
                        final_cable.size_sqmm, final_cable.conduit_size_mm, ground_sqmm
                    )
                } else {
                    format!(
                        "{}x(4x1/C-{} mm² FD-0.6/1kV-CV in HDPE {} mm) + 1-{} mm² IEC01 (Ground)",
                        sets, final_cable.size_sqmm, final_cable.conduit_size_mm, ground_sqmm
                    )
                };

                let cable_spec = CableSpec {
                    description,
                    conductor_size_sqmm: final_cable.size_sqmm,
                    ground_size_sqmm: ground_sqmm,
                    conduit_size_mm: final_cable.conduit_size_mm,
                    sets,
                };

                return CableSizingResult {
                    cable_spec,
                    ampacity_a: final_cable.ampacity_a * (sets as f64),
                    voltage_drop_v: vd_v,
                    voltage_drop_percent: vd_percent,
                    length_m,
                    current_a,
                    upsized_due_to_vd: selected_idx > start_idx || sets > 1,
                };
            }
        }
    }

    // Fallback กรณีโหลดสูงพิเศษ เลือกสายขนาดสูงสุดที่ยอมรับได้ 240 mm²
    let fallback_cable = allowed_cables.last().copied().unwrap_or(&STANDARD_CV_CABLES[0]);
    let (ground_sqmm, _) = select_ground_wire_size_sqmm(fallback_cable.size_sqmm);
    let cable_spec = CableSpec {
        description: format!(
            "4x1/C-{} mm² FD-0.6/1kV-CV in HDPE {} mm + 1-{} mm² IEC01 (Ground)",
            fallback_cable.size_sqmm, fallback_cable.conduit_size_mm, ground_sqmm
        ),
        conductor_size_sqmm: fallback_cable.size_sqmm,
        ground_size_sqmm: ground_sqmm,
        conduit_size_mm: fallback_cable.conduit_size_mm,
        sets: 1,
    };

    CableSizingResult {
        cable_spec,
        ampacity_a: fallback_cable.ampacity_a,
        voltage_drop_v: 0.0,
        voltage_drop_percent: 0.0,
        length_m,
        current_a,
        upsized_due_to_vd: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformer_sizing_for_185_kva() {
        // จากโหลด 185.89 kVA หม้อแปลงมาตรฐาน กฟภ. (160, 250, 315, 500, ...) ต้องเลือกขนาด 250 kVA
        let tr = size_transformer(185.89);
        assert_eq!(tr.kva, 250.0);
        assert_eq!(tr.voltage_pri_kv, 22.0);
        assert_eq!(tr.voltage_sec_v, 400.0);
        assert!((tr.rated_current_a - 360.84).abs() < 1.0);
    }

    #[test]
    fn test_main_and_branch_breaker_selection() {
        // สำหรับกระแสต่อเนื่อง 268.3 A:
        // Target trip = 268.3 * 1.25 = 335.375 A -> ปัดขึ้นเป็น 320AT หรือ 350AT/400AF
        // ตามแบบ SLD กำหนด 320AT / 400AF 3P 50kA
        let breaker = size_main_breaker(255.0); // 255 * 1.25 = 318.75 -> 320AT / 400AF
        assert_eq!(breaker.trip_at, 320);
        assert_eq!(breaker.frame_af, 400);
        assert_eq!(breaker.poles, 3);
        assert_eq!(breaker.icu_ka, 50);

        // EV Branch Breaker
        let ev_breaker = size_ev_branch_breaker(255.0);
        assert_eq!(ev_breaker.trip_at, 320);
        assert_eq!(ev_breaker.frame_af, 400);

        // LP Branch Breaker
        let lp_breaker = size_lp_branch_breaker();
        assert_eq!(lp_breaker.trip_at, 32);
        assert_eq!(lp_breaker.poles, 2);
    }

    #[test]
    fn test_cable_selection_and_grounding_for_268a() {
        // สำหรับกระแส 268 A (ที่ระยะปกติ 25 เมตร):
        // min_ampacity = 268 * 1.25 = 335 A -> สาย 185 mm² (Ampacity 341 A, HDPE 110 mm)
        // สายดินสำหรับสายเฟส 185 mm² คือ 95 mm² (ท่อ 32 mm) ตาม SLD
        let cable_result = size_cable_with_vd(268.0, 25.0, 0.98, 400.0, 2.0);
        assert_eq!(cable_result.cable_spec.conductor_size_sqmm, 185.0);
        assert_eq!(cable_result.cable_spec.ground_size_sqmm, 95.0);
        assert_eq!(cable_result.cable_spec.conduit_size_mm, 110);
        assert!(cable_result.voltage_drop_percent <= 2.0);
        assert!(!cable_result.upsized_due_to_vd);
    }

    #[test]
    fn test_voltage_drop_upsize() {
        // ทดสอบกรณีระยะทางไกล (เช่น 150 เมตร) ทำให้เกิด Voltage Drop > 2.0% สำหรับสาย 185 mm²
        // ระบบจะต้องขยับขนาดสายให้ใหญ่ขึ้นโดยอัตโนมัติ (เป็น 240 mm² หรือเพิ่ม sets สายควบ) แต่ขนาดสายต่อเส้นต้องไม่เกิน 240 mm²
        let cable_result = size_cable_with_vd(268.0, 150.0, 0.98, 400.0, 2.0);
        assert!(cable_result.cable_spec.conductor_size_sqmm <= 240.0);
        assert!(cable_result.voltage_drop_percent <= 2.0);
        assert!(cable_result.upsized_due_to_vd);
    }

    #[test]
    fn test_cable_max_size_is_240_sqmm() {
        // ทดสอบกระแสขนาดใหญ่พิเศษ เช่น 600 A ระบบจะต้องไม่แนะนำสาย 300 หรือ 400 mm²
        // แต่ต้องแบ่งเป็นสายควบ (sets >= 2) โดยสายแต่ละเส้นมีขนาดไม่เกิน 240 mm²
        let cable_result = size_cable_with_vd(600.0, 50.0, 0.98, 400.0, 2.0);
        assert!(cable_result.cable_spec.conductor_size_sqmm <= 240.0);
        assert!(cable_result.cable_spec.sets >= 2);
        assert!(cable_result.voltage_drop_percent <= 2.0);
    }
}
