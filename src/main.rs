use ev_station_calculator::{
    AuxLoad, ChargerType, ChargerUnit, PhaseConnection, StationDesignEngine, StationInput,
};

fn main() {
    println!("============================================================");
    println!("     EV CHARGING STATION ELECTRICAL SIZING CALCULATOR       ");
    println!("         Engineering Engine (PEA / EIT Standard)            ");
    println!("============================================================");

    // 1. จำลองข้อมูลสถานีชาร์จ (ตู้ DC 180kW 1 ตู้ + LP Load 5,888 VA)
    let charger = ChargerUnit {
        id: "EV-01".to_string(),
        name: "DC Fast Charger 180kW".to_string(),
        charger_type: ChargerType::DcFast(180),
        power_kw: 180.0,
        distance_m: 20.0,
        power_factor: 1.0,
        efficiency: 1.0,
    };

    let aux_load = AuxLoad::new("LP Panel (Auxiliary)", 5888.0, PhaseConnection::ThreePhase);

    let input = StationInput {
        chargers: vec![charger],
        aux_loads: vec![aux_load],
        coincidence_factor: 1.0,
        system_voltage_v: Some(400.0),
        max_voltage_drop_percent: Some(2.0),
        main_cable_distance_m: Some(20.0),
    };

    // 2. เรียกใช้ Engine คำนวณระบบไฟฟ้า
    let engine = StationDesignEngine::new();
    match engine.calculate_station(&input) {
        Ok(output) => {
            println!("\n[1] LOAD DEMAND SUMMARY");
            println!("  - Connected Load: {:.2} kVA ({:.0} VA)", 
                output.load_summary.total_connected_kva, output.load_summary.total_connected_va);
            println!("  - Demand Load:    {:.2} kVA ({:.0} VA)", 
                output.load_summary.total_demand_kva, output.load_summary.total_demand_va);
            println!("  - System Voltage: {:.0} V (3-Phase Line-to-Line)", 
                output.load_summary.system_voltage_v);
            println!("  - Max Operating Current: {:.2} A", 
                output.load_summary.max_continuous_current_a);

            println!("\n[2] TRANSFORMER SIZING (กฟภ. 22kV - 400/230V)");
            println!("  - Rating:         {:.0} kVA", output.transformer.kva);
            println!("  - Rated Current:  {:.2} A", output.transformer.rated_current_a);

            println!("\n[3] MAIN DISTRIBUTION BOARD (MDB)");
            println!("  - Main Breaker:   {}", output.main_breaker.name);
            println!("  - Main Feeder:    {}", output.main_cable.cable_spec.description);
            println!("  - Feeder Length:  {:.1} m", output.main_cable.length_m);
            println!("  - Voltage Drop:   {:.2} V ({:.2}%) [Limit <= {:.1}%]", 
                output.main_cable.voltage_drop_v, 
                output.main_cable.voltage_drop_percent,
                input.max_voltage_drop_percent.unwrap_or(2.0));
            println!("  - Auto-upsized:   {}", 
                if output.main_cable.upsized_due_to_vd { "Yes" } else { "No" });

            println!("\n[4] BRANCH CIRCUITS");
            println!("  - LP Auxiliary:   {}", output.lp_branch_breaker.name);
            for (idx, ch_circuit) in output.charger_circuits.iter().enumerate() {
                println!("  - [{}] {} ({})", idx + 1, ch_circuit.charger_name, ch_circuit.charger_id);
                println!("      * Rated Current: {:.2} A", ch_circuit.rated_current_a);
                println!("      * Breaker:       {}", ch_circuit.branch_breaker.name);
                println!("      * Cable Spec:    {}", ch_circuit.cable_sizing.cable_spec.description);
                println!("      * Voltage Drop:  {:.2} V ({:.2}%)", 
                    ch_circuit.cable_sizing.voltage_drop_v, 
                    ch_circuit.cable_sizing.voltage_drop_percent);
            }
            println!("\n============================================================");
            println!("Calculation completed successfully according to SLD & Load Schedule.");
            println!("============================================================");
        }
        Err(err) => {
            eprintln!("Calculation failed: {}", err);
        }
    }
}
