//! Source-grounded modernization contracts, first run against the faithful build.
use super::*;
use crate::{
    frontend::{names, shop},
    race::RaceTimer,
};
use serde_json::json;

fn port() -> Port {
    let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
    Port::load_assets_seeded(&rom, &MemoryAssets::default(), [0; 5]).unwrap()
}

#[test]
fn iteration_three_pickup_wire_states_and_retained_fields_roundtrip() {
    let mut game = port();
    let original: serde_json::Value = serde_json::from_str(&game.snapshot_json().unwrap()).unwrap();
    // Historical version2 accepts the incidental fifth money-award state11,
    // arbitrary count words and a zero award lifetime. Import must not normalize.
    for state in 0..=11 {
        let mut saved = original.clone();
        let pickup = &mut saved["state"]["race"]["world"]["pickup"];
        pickup["state"] = json!(state);
        pickup["counts"] = json!([65535, 4]);
        pickup["lifetime"] = json!(0);
        pickup["sprites"][0]["coordinates"] = json!(65535);
        game.restore_snapshot_json(&saved.to_string()).unwrap();
        let exported: serde_json::Value =
            serde_json::from_str(&game.snapshot_json().unwrap()).unwrap();
        assert_eq!(
            exported["state"]["race"]["world"]["pickup"],
            saved["state"]["race"]["world"]["pickup"]
        );
        assert_eq!(game.observe()["world"]["pickup"]["state"], json!(state));
    }
    let before = game.snapshot_json().unwrap();
    let mut invalid: serde_json::Value = serde_json::from_str(&before).unwrap();
    invalid["state"]["race"]["world"]["pickup"]["state"] = json!(12);
    assert!(game.restore_snapshot_json(&invalid.to_string()).is_err());
    assert_eq!(game.snapshot_json().unwrap(), before);
}

#[test]
fn modernization_legacy_snapshot_preserves_every_nitro_byte_and_unusual_timer() {
    let mut game = port();
    let original: serde_json::Value = serde_json::from_str(&game.snapshot_json().unwrap()).unwrap();
    // Version2 never restricted nitro/timer bytes to canonical decimal. Keeping
    // this boundary is required even though ordinary play uses decimal values.
    for stock in 0..=255 {
        let mut saved = original.clone();
        saved["state"]["race"]["trucks"][0]["nitro_stock_bcd"] = json!(stock);
        saved["state"]["race"]["timer"] = json!({"fraction_bcd":250,"middle_bcd":0,"high_bcd":241});
        game.restore_snapshot_json(&saved.to_string()).unwrap();
        let exported: serde_json::Value =
            serde_json::from_str(&game.snapshot_json().unwrap()).unwrap();
        assert_eq!(
            exported["state"]["race"]["trucks"][0]["nitro_stock_bcd"],
            json!(stock)
        );
        assert_eq!(
            exported["state"]["race"]["timer"],
            saved["state"]["race"]["timer"]
        );
        assert_eq!(game.observe()["actors"][0]["nitro"], json!(stock));
    }
}

#[test]
fn modernization_conditional_timer_carry_keeps_existing_byte_semantics() {
    // race.rs decimal_add follows the established faithful invalid-digit policy;
    // do not turn this into an unmeasured claim about invalid hardware BCD.
    let mut time: RaceTimer =
        serde_json::from_value(json!({"fraction_bcd":250,"middle_bcd":0,"high_bcd":0})).unwrap();
    crate::race::advance_timer(&mut time);
    assert_eq!(
        serde_json::to_value(time).unwrap(),
        json!({"fraction_bcd":118,"middle_bcd":1,"high_bcd":0})
    );
}

#[test]
fn modernization_frontend_handoffs_preserve_nmi_yield_and_window_owner() {
    let mut game = port();
    game.state.session.page_limit = 0;
    game.begin_names();
    game.state.session.joined_pages = [1, 0];
    let state::Menu::Names(selection) = &mut game.state.menu else {
        panic!()
    };
    selection.phase = names::Phase::FinalWait;
    let inhibit = game.state.session.inhibit_video;
    game.tick(&InputState::default());
    assert!(game.state.window.is_none());
    assert_eq!(game.state.session.inhibit_video, inhibit.wrapping_add(1));
    assert!(
        matches!(&game.state.menu, state::Menu::Names(s) if s.phase == names::Phase::ContractWindow)
    );
    // Names completion installs Shop but yields before its first fade service. This is
    // the source frontend return boundary; Shop completion instead falls through
    // to race preparation during the same NMI.
    for _ in 0..500 {
        game.tick(&InputState::default());
        assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
        if game.state.scene == Scene::Shop {
            break;
        }
    }
    assert_eq!(game.state.scene, Scene::Shop);
    assert!(game.state.window.is_some());
    assert!(
        matches!(&game.state.menu, state::Menu::Shop(s) if s.phase == shop::Phase::FadeIn && !s.fade.as_ref().unwrap().primed)
    );
    game.tick(&InputState::default());
    assert!(matches!(&game.state.menu, state::Menu::Shop(s) if s.phase == shop::Phase::FadeIn));
    let state::Menu::Shop(s) = &mut game.state.menu else {
        panic!()
    };
    s.phase = shop::Phase::Complete;
    game.tick(&InputState::default());
    assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
    assert_eq!(game.state.scene, Scene::RaceFade);
    assert_eq!(
        serde_json::to_value(game.state.race.timer).unwrap(),
        json!({"fraction_bcd":22,"middle_bcd":0,"high_bcd":0})
    );
}

#[test]
fn modernization_conditional_nitro_consumption_preserves_legacy_vectors() {
    use crate::{
        controls::{BUTTON_Y, ControllerWord},
        trucks::motion::{self, Truck},
    };
    for (encoded, expected) in [
        (0x00, 0x00),
        (0x0a, 0x09),
        (0x10, 0x09),
        (0xf0, 0xe9),
        (0xff, 0xfe),
    ] {
        let mut saved = serde_json::to_value(Truck::default()).unwrap();
        saved["nitro_stock_bcd"] = json!(encoded);
        let mut truck: Truck = serde_json::from_value(saved).unwrap();
        truck.controller = ControllerWord(BUTTON_Y);
        motion::use_nitro(&mut truck, false, 1);
        assert_eq!(
            serde_json::to_value(&truck).unwrap()["nitro_stock_bcd"],
            json!(expected)
        );
    }
}
