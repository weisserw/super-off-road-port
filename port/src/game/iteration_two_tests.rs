//! Iteration2 contracts run on the starting implementation before conversion.
use crate::{
    runtime::RandomState,
    trucks::{ai, bearings::BearingTable, motion::Truck},
    world::{objects, tracks},
};
use serde_json::json;

#[test]
fn iteration_two_bearing_loaders_preserve_every_overridden_cell() {
    let mut rom = std::fs::read("../input/game.rom").unwrap();
    for (index, value) in rom[0x1cc7..0x1dc7].iter_mut().enumerate() {
        *value = index as u8 ^ 0xa5;
    }
    let ai = ai::load_tables(&rom).unwrap();
    let contact = BearingTable::load(&rom).unwrap();
    assert_eq!(ai.bearings.cells.as_slice(), &rom[0x1cc7..0x1dc7]);
    assert_eq!(contact, ai.bearings);
}

#[test]
fn iteration_two_steering_discards_upper_coordinate_bits_after_source_reduction() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let tables = ai::load_tables(&rom).unwrap();
    // Source subtracts, shifts6 and keeps a byte: only bits6..13 reach the
    // quantizer. Bit14/15 affect acceptance/ranking but cannot carry back down
    // into steering. This corrects the menu's overly cautious bit14 carry claim.
    for bias in [0, 1, 3, 0xfc, 0xff] {
        let truck = Truck {
            world: [0x2020, 0x3120],
            ai_route_bias: bias,
            ..Truck::default()
        };
        let expected = ai::desired_heading(
            &truck,
            tracks::Waypoint::from_encoded([0x1004, 0x2004]),
            &tables,
        );
        for upper in [0, 0x4000, 0x8000, 0xc000] {
            assert_eq!(
                ai::desired_heading(
                    &truck,
                    tracks::Waypoint::from_encoded([0x1004 | upper, 0x2004 | upper]),
                    &tables
                ),
                expected
            );
        }
    }
}

#[test]
fn iteration_two_original_redirects_keep_bias_then_branch_rng_and_wire_identity() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    for (track, direction, conditional, unconditional, target) in [
        (8, tracks::Direction::Forward, 3, 29, 13),
        (13, tracks::Direction::Forward, 3, 22, 9),
        (8, tracks::Direction::Reversed, 4, 30, 14),
        (13, tracks::Direction::Reversed, 6, 26, 12),
    ] {
        let route = tracks::load_route(&rom, track, direction).unwrap();
        let encoded = serde_json::to_value(&route).unwrap();
        let restored: tracks::Route = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(serde_json::to_value(restored).unwrap(), encoded);
        for (node, expected, history) in [
            (conditional, conditional + 1, [0x6a, 0xdf, 0x12, 0x34, 0x56]),
            (unconditional, target, [0xdf, 0x12, 0x34, 0x56, 0x78]),
        ] {
            let mut driver = ai::Driver {
                checkpoint: node - 1,
                ..Default::default()
            };
            let mut truck = Truck::default();
            let mut random = RandomState([0x12, 0x34, 0x56, 0x78, 0x9b]);
            ai::next_target(&mut truck, &mut driver, &route, &mut random).unwrap();
            assert_eq!(
                (driver.checkpoint, truck.ai_route_bias, driver.bias_delay),
                (expected, 3, 100)
            );
            assert_eq!(random.0, history);
        }
        let mut driver = ai::Driver {
            checkpoint: conditional - 1,
            ..Default::default()
        };
        let mut truck = Truck::default();
        ai::next_target(&mut truck, &mut driver, &route, &mut RandomState([0; 5])).unwrap();
        assert_eq!(
            driver.checkpoint,
            if track == 8 && direction == tracks::Direction::Forward {
                20
            } else if track == 13 && direction == tracks::Direction::Forward {
                17
            } else {
                21
            }
        );
    }
}

fn hazard(kind: u16, timer: u16, uses: u16, animation: u16) -> objects::DynamicObject {
    serde_json::from_value(json!({"kind":kind,"placement":{"world":[4096,4096],"height":0},"animation":animation,"timer":timer,"uses":uses,"sprite":{"coordinates":16400,"tile_attributes":9504}})).unwrap()
}

#[test]
fn iteration_two_hazard_response_depletion_and_struck_transition_preserve_order() {
    let rom = std::fs::read("../input/game.rom").unwrap();
    let tables = objects::load_object_tables(&rom).unwrap();
    let mut hazards = std::array::from_fn(|_| objects::DynamicObject::default());
    hazards[5] = hazard(1, 0, 0, 0);
    let mut trucks = std::array::from_fn(|_| Truck {
        world: [4096, 4096],
        speed: 0xab30,
        ..Default::default()
    });
    assert_eq!(
        objects::service_dynamic(&mut hazards, &mut trucks, 1, 1, &tables).unwrap(),
        [objects::DYNAMIC_CONTACT_SOUND]
    );
    let first = serde_json::to_value(&hazards[5]).unwrap();
    assert_eq!(trucks[1].speed, 0);
    assert_eq!(first["timer"], 43);
    assert_eq!(first["animation"], 34);
    for _ in 0..43 {
        objects::service_dynamic(&mut hazards, &mut trucks, 1, 1, &tables).unwrap();
    }
    assert_eq!(serde_json::to_value(&hazards[5]).unwrap()["timer"], 0);
    objects::service_dynamic(&mut hazards, &mut trucks, 1, 1, &tables).unwrap();
    let depleted = serde_json::to_value(&hazards[5]).unwrap();
    assert_eq!(depleted["kind"], 2);
    assert_eq!(depleted["uses"], 65535);
    assert_eq!(
        depleted["sprite"]["tile_attributes"],
        objects::DYNAMIC_EXHAUSTED_TILE
    );
    objects::service_dynamic(&mut hazards, &mut trucks, 1, 1, &tables).unwrap();
    assert_eq!(serde_json::to_value(&hazards[5]).unwrap()["kind"], 3);
    for _ in 0..17 {
        objects::service_dynamic(&mut hazards, &mut trucks, 0, 1, &tables).unwrap();
    }
    trucks[0].speed = 0x8000;
    objects::service_dynamic(&mut hazards, &mut trucks, 0, 1, &tables).unwrap();
    assert_eq!(serde_json::to_value(&hazards[5]).unwrap()["animation"], 28);
    assert_eq!(trucks[0].speed, 0x8000);
}

#[test]
fn iteration_two_historical_hazard_state_combinations_round_trip_losslessly() {
    for kind in 0..4 {
        for timer in [0, 1, 44, 0x8000, 0xffff] {
            for uses in [0, 1, 7, 0x7fff, 0x8000, 0xffff] {
                for animation in [0, 18, 29, 34] {
                    let object = hazard(kind, timer, uses, animation);
                    let encoded = serde_json::to_value(&object).unwrap();
                    let restored: objects::DynamicObject =
                        serde_json::from_value(encoded.clone()).unwrap();
                    assert_eq!(serde_json::to_value(restored).unwrap(), encoded);
                }
            }
        }
    }
}
