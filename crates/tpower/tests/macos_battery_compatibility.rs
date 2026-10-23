use tpower::{
    de::{repr, BatteryData, IORegistry},
    ffi::smc::SMCPowerData,
    provider::NormalizedResource,
};

#[test]
fn missing_raw_battery_fields_do_not_reject_available_readings() {
    // Synthetic payload: retain ordinary readings while omitting raw-capacity
    // and temperature keys that newer macOS versions may no longer expose.
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <plist version="1.0"><dict>
            <key>AdapterDetails</key><dict/>
            <key>CurrentCapacity</key><integer>80</integer>
            <key>MaxCapacity</key><integer>100</integer>
            <key>Amperage</key><integer>-1500</integer>
            <key>Voltage</key><integer>12000</integer>
            <key>CycleCount</key><integer>100</integer>
            <key>IsCharging</key><false/>
            <key>TimeRemaining</key><integer>180</integer>
            <key>UpdateTime</key><integer>1700000000</integer>
        </dict></plist>"#;

    let io: repr::IORegistry = plist::from_bytes(xml).unwrap();
    assert_eq!(io.current_capacity, 80);
    assert_eq!(io.amperage, -1500);
    assert_eq!(io.voltage, 12000);
    assert_eq!(io.time_remaining, 180);
}

#[test]
fn missing_raw_capacity_keeps_normalized_battery_level_finite() {
    let io = IORegistry {
        current_capacity: 80,
        max_capacity: 100,
        ..Default::default()
    };
    let smc = SMCPowerData::default();

    for resource in [
        NormalizedResource::from(&io),
        NormalizedResource::from((&io, &smc)),
    ] {
        assert_eq!(resource.battery_level, 80);
        assert_eq!(resource.current_capacity, 0);
        assert_eq!(resource.max_capacity, 0);
        assert_eq!(resource.design_capacity, 0);
        assert!(resource.absolute_battery_level.is_finite());
    }
}

#[test]
fn available_raw_capacity_still_produces_the_correct_percentage() {
    let io = IORegistry {
        current_capacity: 80,
        apple_raw_current_capacity: 4000,
        apple_raw_max_capacity: 5000,
        design_capacity: 6000,
        ..Default::default()
    };
    let smc = SMCPowerData::default();

    for resource in [
        NormalizedResource::from(&io),
        NormalizedResource::from((&io, &smc)),
    ] {
        assert_eq!(resource.current_capacity, 4000);
        assert_eq!(resource.max_capacity, 5000);
        assert_eq!(resource.design_capacity, 6000);
        assert!((resource.absolute_battery_level - 80.0).abs() < 0.001);
    }
}

#[test]
fn nested_battery_data_supplies_mah_to_both_normalization_paths() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
        <plist version="1.0"><dict>
            <key>CurrentCapacity</key><integer>80</integer>
            <key>MaxCapacity</key><integer>100</integer>
            <key>BatteryData</key><dict>
                <key>CurrentCapacity</key><integer>80</integer>
                <key>MaxCapacity</key><integer>100</integer>
                <key>RemainingCapacity</key><integer>4000</integer>
                <key>FullChargeCapacity</key><integer>5000</integer>
                <key>DesignCapacity</key><integer>6000</integer>
            </dict>
        </dict></plist>"#;
    let parsed: repr::IORegistry = plist::from_bytes(xml).unwrap();
    let io: IORegistry = parsed.into();
    let smc = SMCPowerData::default();

    for resource in [
        NormalizedResource::from(&io),
        NormalizedResource::from((&io, &smc)),
    ] {
        assert_eq!(resource.battery_level, 80);
        assert_eq!(resource.current_capacity, 4000);
        assert_eq!(resource.max_capacity, 5000);
        assert_eq!(resource.design_capacity, 6000);
        assert!((resource.absolute_battery_level - 80.0).abs() < 0.001);
    }
}

#[test]
fn partial_nested_battery_data_preserves_legacy_capacity_fields() {
    let io = IORegistry {
        apple_raw_current_capacity: 4000,
        apple_raw_max_capacity: 5000,
        design_capacity: 6000,
        battery_data: Some(BatteryData {
            full_charge_capacity: Some(5500),
            ..Default::default()
        }),
        ..Default::default()
    };
    let smc = SMCPowerData::default();

    for resource in [
        NormalizedResource::from(&io),
        NormalizedResource::from((&io, &smc)),
    ] {
        assert_eq!(resource.current_capacity, 4000);
        assert_eq!(resource.max_capacity, 5500);
        assert_eq!(resource.design_capacity, 6000);
    }
}

#[test]
fn zero_remaining_capacity_is_valid_but_negative_measurements_fall_back() {
    for (remaining, expected) in [(Some(0), 0), (Some(-1), 4000), (None, 4000)] {
        let io = IORegistry {
            apple_raw_current_capacity: 4000,
            apple_raw_max_capacity: 5000,
            design_capacity: 6000,
            battery_data: Some(BatteryData {
                remaining_capacity: remaining,
                full_charge_capacity: Some(0),
                design_capacity: Some(-1),
            }),
            ..Default::default()
        };
        let smc = SMCPowerData::default();

        for resource in [
            NormalizedResource::from(&io),
            NormalizedResource::from((&io, &smc)),
        ] {
            assert_eq!(resource.current_capacity, expected);
            assert_eq!(resource.max_capacity, 5000);
            assert_eq!(resource.design_capacity, 6000);
        }
    }
}

#[test]
fn charge_limit_hold_suppresses_the_time_to_full_sentinel() {
    let io = IORegistry {
        external_connected: true,
        is_charging: false,
        amperage: 0,
        current_capacity: 90,
        time_remaining: 65535,
        ..Default::default()
    };
    let smc = SMCPowerData {
        charging_status: 1.0,
        time_to_full: 65535.0,
        ..Default::default()
    };

    for resource in [
        NormalizedResource::from(&io),
        NormalizedResource::from((&io, &smc)),
    ] {
        assert!(resource.is_charging_paused);
        assert_eq!(resource.battery_level, 90);
        assert_eq!(resource.time_remain, std::time::Duration::ZERO);
    }
}

#[test]
fn active_charging_discharging_and_full_states_are_not_paused() {
    for (external, charging, full, amperage) in [
        (true, true, false, 500),
        (true, false, false, -500),
        (false, false, false, -500),
        (true, false, true, 0),
    ] {
        let io = IORegistry {
            external_connected: external,
            is_charging: charging,
            fully_charged: full,
            amperage,
            time_remaining: 30,
            ..Default::default()
        };
        let smc = SMCPowerData {
            charging_status: if charging { 1.0 } else { 0.0 },
            time_to_full: 30.0,
            time_to_empty: 30.0,
            ..Default::default()
        };

        for resource in [
            NormalizedResource::from(&io),
            NormalizedResource::from((&io, &smc)),
        ] {
            assert!(!resource.is_charging_paused);
            assert_eq!(resource.time_remain, std::time::Duration::from_secs(1800));
        }
    }
}

#[test]
fn unknown_remaining_time_values_do_not_become_countdowns() {
    for minutes in [-1, 65535] {
        let io = IORegistry {
            time_remaining: minutes,
            ..Default::default()
        };
        assert_eq!(
            NormalizedResource::from(&io).time_remain,
            std::time::Duration::ZERO
        );
    }
    for minutes in [-1.0, 65535.0, f32::NAN, f32::INFINITY] {
        let io = IORegistry::default();
        let smc = SMCPowerData {
            time_to_empty: minutes,
            ..Default::default()
        };
        assert_eq!(
            NormalizedResource::from((&io, &smc)).time_remain,
            std::time::Duration::ZERO
        );
    }
}
