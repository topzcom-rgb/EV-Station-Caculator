use ev_station_calculator::{
    AuxLoad, ChargerType, ChargerUnit, PhaseConnection, StationDesignEngine, StationInput,
};

#[test]
fn test_station_reference_against_sld_and_load_schedule() {
    // กำหนดข้อมูลการทดสอบ:
    // - ตู้ DC Fast Charger 180kW (1 ตู้, ระยะทาง 20 ม., pf=1.0, eff=1.0 เพื่อตรงตาม Load Schedule)
    // - ตู้ LP Load 5,888 VA (3 เฟส)
    let charger = ChargerUnit {
        id: "EV-01".to_string(),
        name: "DC Fast Charger 180kW".to_string(),
        charger_type: ChargerType::DcFast(180),
        power_kw: 180.0,
        distance_m: 20.0,
        power_factor: 1.0,
        efficiency: 1.0,
    };

    let aux_load = AuxLoad::new("LP Panel", 5888.0, PhaseConnection::ThreePhase);

    let input = StationInput {
        chargers: vec![charger],
        aux_loads: vec![aux_load],
        coincidence_factor: 1.0,
        system_voltage_v: Some(400.0),
        max_voltage_drop_percent: Some(2.0),
        main_cable_distance_m: Some(20.0),
    };

    let engine = StationDesignEngine::new();
    let result = engine.calculate_station(&input);

    assert!(result.is_ok(), "Engine calculation should succeed");
    let output = result.unwrap();

    // 1. หม้อแปลงที่ได้ = 250 kVA
    assert_eq!(
        output.transformer.kva, 250.0,
        "Transformer should be 250 kVA"
    );

    // 2. Main Breaker = 320AT / 400AF 3P 50kA
    assert_eq!(
        output.main_breaker.trip_at, 320,
        "Main breaker Trip should be 320AT"
    );
    assert_eq!(
        output.main_breaker.frame_af, 400,
        "Main breaker Frame should be 400AF"
    );
    assert_eq!(output.main_breaker.poles, 3, "Main breaker should be 3-Pole");
    assert_eq!(
        output.main_breaker.icu_ka, 50,
        "Main breaker Icu should be 50kA"
    );

    // 3. สายเมน = 185 mm² FD-CV
    assert_eq!(
        output.main_cable.cable_spec.conductor_size_sqmm, 185.0,
        "Main conductor should be 185 mm²"
    );

    // 4. สายดิน = 95 mm² IEC01
    assert_eq!(
        output.main_cable.cable_spec.ground_size_sqmm, 95.0,
        "Ground wire should be 95 mm²"
    );

    // 5. ขนาดท่อเมน = 110 mm HDPE
    assert_eq!(
        output.main_cable.cable_spec.conduit_size_mm, 110,
        "Main conduit should be 110 mm"
    );

    // ตรวจสอบเพิ่มเติมสำหรับวงจรย่อย EV
    assert_eq!(output.charger_circuits.len(), 1);
    let ev_circuit = &output.charger_circuits[0];
    assert_eq!(ev_circuit.branch_breaker.trip_at, 320);
    assert_eq!(ev_circuit.branch_breaker.frame_af, 400);
    assert_eq!(ev_circuit.cable_sizing.cable_spec.conductor_size_sqmm, 185.0);
    assert_eq!(ev_circuit.cable_sizing.cable_spec.ground_size_sqmm, 95.0);
    assert_eq!(ev_circuit.cable_sizing.cable_spec.conduit_size_mm, 110);
    assert!(ev_circuit.cable_sizing.voltage_drop_percent <= 2.0);
}
