use super::*;

#[test]
fn player_loading_seeds_a_live_rng_and_snapshots_retain_the_reset_history() {
    let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
    let mut game = Port::load_assets(&rom, &MemoryAssets::default()).unwrap();
    let initial = game.observe();
    let mut random = game.state.random;
    let draws: Vec<_> = (0..8)
        .map(|_| random.next(crate::runtime::RandomResultWidth::Byte))
        .collect();
    assert!(
        draws.iter().any(|&value| value != 0),
        "power-on entropy must not enter the absorbing zero RNG state"
    );
    let saved = game.snapshot_json().unwrap();
    let mut receiver = port();
    receiver.restore_snapshot_json(&saved).unwrap();
    receiver.reset().unwrap();
    assert_eq!(
        receiver.observe(),
        initial,
        "restoring a save retains its original random reset history"
    );
    game.tick(&InputState::default());
    game.reset().unwrap();
    assert_eq!(
        game.observe(),
        initial,
        "reset within a loaded game remains replayable"
    );
}

#[test]
fn driving_with_varied_seeds_keeps_running_and_can_save_audio() {
    use crate::frontend::shop;
    let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
    for seed in [
        [18, 52, 86, 120, 155],
        [245, 252, 226, 140, 107],
        [83, 248, 138, 90, 55],
    ] {
        let mut game = Port::load_assets_seeded(&rom, &MemoryAssets::default(), seed).unwrap();
        game.state.session.records = game.resources.results.defaults.clone();
        shop::initialize_wallets(&mut game.state.session);
        game.state.session.active_humans = 1;
        game.state.session.joined_pages = [1, 0];
        game.state.scene = Scene::RacePrepare;
        let mut clock = romatlas_port::audio::SampleClock::new(crate::config::RATE, 48000).unwrap();
        for frame in 0..12000 {
            let mut input = InputState::default();
            if game.state.scene == Scene::Racing {
                input.held.insert("b".into());
                if frame % 240 < 120 {
                    input.held.insert("left".into());
                }
                if frame % 90 < 5 {
                    input.held.insert("y".into());
                }
            }
            game.tick(&input);
            let mut audio = Vec::new();
            game.audio(clock.next_tick(), 48000, &mut audio);
            assert!(
                game.state.fault.is_none(),
                "seed {seed:?}, frame {frame}: {:?}",
                game.state.fault
            );
            if frame % 120 == 0 {
                let saved = game.snapshot_json().unwrap();
                let mut restored = game.clone();
                restored.restore_snapshot_json(&saved).unwrap();
                let mut branch = game.sound.clone();
                assert_eq!(
                    branch.render_output(32, 48000).unwrap(),
                    restored.sound.render_output(32, 48000).unwrap()
                );
            }
            if game.state.scene == Scene::Shop || game.state.scene == Scene::LossPrompt {
                break;
            }
        }
    }
}

#[test]
fn computer_can_finish_a_natural_race_and_enter_results() {
    use crate::frontend::{results, shop};
    let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
    let mut game = Port::load_assets_seeded(
        &rom,
        &MemoryAssets::default(),
        [0x12, 0x34, 0x56, 0x78, 0x9b],
    )
    .unwrap();
    game.state.session.records = game.resources.results.defaults.clone();
    shop::initialize_wallets(&mut game.state.session);
    game.state.session.active_humans = 1;
    game.state.session.joined_pages = [1, 0];
    game.state.scene = Scene::RacePrepare;
    let mut clock = romatlas_port::audio::SampleClock::new(crate::config::RATE, 48000).unwrap();
    for frame in 0..12000 {
        game.tick(&InputState::default());
        let mut audio = Vec::new();
        game.audio(clock.next_tick(), 48000, &mut audio);
        assert!(
            game.state.fault.is_none(),
            "frame {frame}, scene {:?}: {:?}",
            game.state.scene,
            game.state.fault
        );
        if game
            .state
            .postrace
            .as_ref()
            .is_some_and(|postrace| postrace.phase == results::PostracePhase::Awards)
        {
            return;
        }
    }
    panic!(
        "race did not finish: scene {:?}, progress {:?}",
        game.state.scene, game.state.race.progress
    );
}

#[test]
fn crossing_the_final_human_checkpoint_advances_to_results() {
    use crate::frontend::{results, shop};
    let mut game = port();
    game.state.session.records = game.resources.results.defaults.clone();
    shop::initialize_wallets(&mut game.state.session);
    game.state.session.active_humans = 1;
    game.state.session.joined_pages = [1, 0];
    game.state.scene = Scene::RacePrepare;
    for _ in 0..1000 {
        game.tick(&InputState::default());
        if game.state.scene == Scene::Racing {
            break;
        }
    }
    let route = &game.selected.track.routes[usize::from(game.selected.reversed)];
    game.state.race.trucks[0].world = route
        .node(0)
        .unwrap()
        .waypoint()
        .unwrap()
        .axes
        .map(|axis| axis.center);
    game.state.race.trucks[0].speed = 0;
    game.state.race.progress.drivers[0].checkpoint = 0;
    game.state.race.progress.drivers[0].lap = crate::race::WINNING_LAP - 1;
    game.state.race.progress.banner_state = crate::race::LAST_LAP_BANNER;
    let mut clock = romatlas_port::audio::SampleClock::new(crate::config::RATE, 48000).unwrap();
    for _ in 0..1200 {
        game.tick(&InputState::default());
        let mut audio = Vec::new();
        game.audio(clock.next_tick(), 48000, &mut audio);
        assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
        if game
            .state
            .postrace
            .as_ref()
            .is_some_and(|postrace| postrace.phase == results::PostracePhase::Awards)
        {
            assert_eq!(game.state.race.progress.winner_plus_one, 1);
            return;
        }
    }
    panic!(
        "finish-line crossing stalled in {:?}: {:?}",
        game.state.scene, game.state.race.progress
    );
}

#[test]
fn finishing_a_race_reaches_the_podium_and_next_race_without_faulting() {
    use crate::frontend::{results, shop};
    for (winner, human_loses) in [(0, false), (1, false), (1, true)] {
        let mut game = port();
        game.state.session.records = game.resources.results.defaults.clone();
        shop::initialize_wallets(&mut game.state.session);
        game.state.session.active_humans = 1;
        game.state.session.joined_pages = [1, 0];
        game.state.scene = Scene::RacePrepare;
        for _ in 0..1000 {
            game.tick(&InputState::default());
            assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
            if game.state.scene == Scene::Racing {
                break;
            }
        }
        assert_eq!(game.state.scene, Scene::Racing);
        // Supply the finish-line outcome, then exercise the actual shared Game
        // dispatcher and every NMI/window/fade between racing and the podium.
        game.state.race.progress.winner_plus_one = winner + 1;
        if human_loses {
            for (driver, lap) in game
                .state
                .race
                .progress
                .drivers
                .iter_mut()
                .zip([1, 5, 4, 3])
            {
                driver.lap = lap;
            }
        }
        game.state.race.progress.drivers[usize::from(winner)].lap = crate::race::WINNING_LAP;
        let mut clock = romatlas_port::audio::SampleClock::new(crate::config::RATE, 48000).unwrap();
        let mut reached_awards = false;
        for _ in 0..2000 {
            game.tick(&InputState::default());
            let mut samples = Vec::new();
            game.audio(clock.next_tick(), 48000, &mut samples);
            assert!(
                game.state.fault.is_none(),
                "winner {winner}: {:?}; transition {:?}",
                game.state.fault,
                game.state.transition
            );
            if game
                .state
                .postrace
                .as_ref()
                .is_some_and(|postrace| postrace.phase == results::PostracePhase::Awards)
            {
                if !reached_awards {
                    let mut frame =
                        Frame::new(crate::config::SIZE.0, crate::config::SIZE.1).unwrap();
                    game.render(&mut frame);
                    assert!(
                        frame
                            .pixels
                            .chunks_exact(4)
                            .any(|pixel| pixel[..3] != [0, 0, 0]),
                        "podium must be visible after its fade-in"
                    );
                }
                reached_awards = true;
            }
            if game.state.scene == Scene::Shop || game.state.scene == Scene::LossPrompt {
                break;
            }
        }
        assert!(reached_awards);
        assert!(
            matches!(game.state.scene, Scene::Shop | Scene::LossPrompt),
            "winner {winner}: scene {:?}, postrace {:?}, transition {:?}",
            game.state.scene,
            game.state.postrace.as_ref().map(|postrace| postrace.phase),
            game.state.transition
        );
        let tick = |game: &mut Port,
                    actions: &[&str],
                    ticks: usize,
                    clock: &mut romatlas_port::audio::SampleClock| {
            let mut input = InputState::default();
            input
                .held
                .extend(actions.iter().map(|action| (*action).into()));
            for _ in 0..ticks {
                game.tick(&input);
                let mut samples = Vec::new();
                game.audio(clock.next_tick(), 48000, &mut samples);
                assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
            }
        };
        if human_loses {
            assert_eq!(game.state.scene, Scene::LossPrompt);
            tick(&mut game, &["b"], 1, &mut clock);
        } else {
            assert_eq!(game.state.scene, Scene::Shop);
        }
        for _ in 0..500 {
            if matches!(&game.state.menu, state::Menu::Shop(shop) if shop.phase == shop::Phase::WaitFrame)
            {
                break;
            }
            tick(&mut game, &[], 1, &mut clock);
        }
        tick(&mut game, &["down"], 25, &mut clock);
        tick(&mut game, &[], 4, &mut clock);
        tick(&mut game, &["right"], 4, &mut clock);
        tick(&mut game, &[], 4, &mut clock);
        tick(&mut game, &["b"], 8, &mut clock);
        for _ in 0..1000 {
            if game.state.scene == Scene::Racing && game.state.sequence_index == 1 {
                break;
            }
            tick(&mut game, &[], 1, &mut clock);
        }
        assert_eq!(game.state.sequence_index, 1);
        assert_eq!(game.state.scene, Scene::Racing);
    }
}
fn port() -> Port {
    let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
    Port::load_assets_seeded(
        &rom,
        &MemoryAssets::default(),
        [0; crate::runtime::RNG_STATE_BYTES],
    )
    .unwrap()
}
#[test]
fn full_snapshot_restore_is_atomic_and_clone_isolated() {
    let mut game = port();
    game.tick(&InputState::default());
    let saved = game.snapshot_json().unwrap();
    let branch = game.clone();
    game.tick(&InputState::default());
    assert_ne!(game.observe(), branch.observe());
    assert!(
        game.restore_snapshot_json(r#"{"version":1,"rom_sha256":"bad","simulation_ticks":9}"#)
            .is_err()
    );
    assert_eq!(game.observe()["simulation_ticks"], 2);
    game.restore_snapshot_json(&saved).unwrap();
    assert_eq!(game.observe(), branch.observe());
    game.reset().unwrap();
    assert_eq!(game.observe()["simulation_ticks"], 0);
}
#[test]
fn debug_checkpoint_and_branch_restore_preserve_observations_and_hashes() {
    use serde_json::json;
    let requests = vec![
        ("port.advance", json!({"ticks":3})),
        ("port.checkpoint.export", json!({})),
        ("port.snapshot.save", json!({"name":"branch"})),
        ("port.state", json!({})),
        ("port.advance", json!({"ticks":4})),
        ("port.snapshot.load", json!({"name":"branch"})),
        ("port.state", json!({})),
    ];
    let run = |requests: Vec<(&str, serde_json::Value)>| {
        let input = requests
            .into_iter()
            .enumerate()
            .map(|(id, (method, params))| {
                json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}).to_string() + "\n"
            })
            .collect::<String>();
        let mut output = Vec::new();
        romatlas_port::debug::serve_io_with_snapshots(
            &mut input.as_bytes(),
            &mut output,
            || Ok(port()),
            crate::config::SIZE,
            crate::config::RATE,
            Default::default(),
            "skeleton-test".into(),
            Port::observe,
        )
        .unwrap();
        String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
            .collect::<Vec<_>>()
    };
    let replies = run(requests);
    for reply in &replies {
        assert!(reply.get("error").is_none(), "{reply}");
    }
    assert_eq!(replies[3]["result"], replies[6]["result"]);
    let checkpoint = replies[1]["result"].clone();
    let restored = run(vec![
        // Replay includes resource loading/rendering; allow slower unoptimized
        // test builds without changing the production debugger's default.
        (
            "port.checkpoint.import",
            json!({"checkpoint":checkpoint,"timeout_ms":30000}),
        ),
        ("port.state", json!({})),
    ]);
    assert!(restored[0].get("error").is_none(), "{}", restored[0]);
    assert_eq!(replies[3]["result"], restored[1]["result"]);
}

#[test]
fn pending_join_snapshot_replays_sampled_input_active_audio_frame_and_owner_state() {
    use super::state::Menu;
    use crate::frontend::names;
    let mut game = port();
    game.state.session.signature = game.resources.frontend.signature;
    game.state.session.records = game.resources.results.defaults.clone();
    crate::frontend::shop::initialize_wallets(&mut game.state.session);
    game.state.arcade.accepted = true;
    game.state.arcade.random = false;
    game.begin_names();
    let Menu::Names(selection) = &mut game.state.menu else {
        panic!("names owner")
    };
    names::initialize_page(selection, &mut game.state.session, &game.resources.names);
    names::install_name_page(
        selection,
        &game.resources.names,
        &game.resources.shared,
        &mut game.state.race.video.memory,
        &mut game.state.race.video.display,
        &mut game.state.race.video.sprites,
    )
    .unwrap();
    selection.phase = names::Phase::WaitFrame;
    selection.sequence = None;
    game.state.session.inhibit_video = 0;
    game.state.race.video.display.force_blank = false;
    game.state.race.video.display.brightness = 15;
    game.state.race.video.display.main_layers = 0x15;
    game.sync_race_views();
    let mut input = MultiInputState::default();
    input.players[0].held.insert("select".into());
    game.tick_players(&input);
    assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
    assert_eq!(game.state.menu.selection().phase, names::Phase::JoinDraw(0));
    assert_eq!(game.state.session.joined_pages, [1, 0]);
    game.send_sound(0x0102).unwrap();
    game.sound.host.second_update_samples = Some(264);
    let checkpoint = game.snapshot_json().unwrap();
    let mut branch = game.clone();
    input.players[0].held.clear();
    input.players[0].held.insert("b".into());
    let run = |game: &mut Port| {
        game.tick_players(&input);
        let mut audio = Vec::new();
        game.audio(700, 32000, &mut audio);
        let mut frame = Frame::new(256, 224).unwrap();
        game.render(&mut frame);
        (game.observe(), frame.pixels, audio)
    };
    let expected = run(&mut branch);
    assert!(
        expected.2.iter().any(|sample| *sample != 0.0),
        "native effect must synthesize active PCM"
    );
    assert_eq!(
        branch.state.menu.selection().editors[0].cursor_byte,
        0,
        "pending join resumes original Select sample, not new B"
    );
    assert_eq!(
        game.snapshot_json().unwrap(),
        checkpoint,
        "branch cannot mutate live owners"
    );
    assert_eq!(run(&mut game), expected);
    game.restore_snapshot_json(&checkpoint).unwrap();
    assert_eq!(run(&mut game), expected);
}

#[test]
fn malformed_nested_snapshot_rejections_preserve_every_live_owner_and_audio() {
    let mut game = port();
    game.tick(&InputState::default());
    let checkpoint = game.snapshot_json().unwrap();
    let original = game.observe();
    for path in ["score", "window", "editor", "scene", "sound"] {
        let mut candidate: serde_json::Value = serde_json::from_str(&checkpoint).unwrap();
        match path {
            "score" => {
                candidate["state"]["session"]["records"]["score_order"][9] = serde_json::json!(0)
            }
            "window" => candidate["state"]["window"]["buffers"][1] = serde_json::json!([]),
            "editor" => {
                candidate["state"]["menu"]["Idle"]["editors"][0]["cursor_byte"] =
                    serde_json::json!(7)
            }
            "scene" => candidate["state"]["scene"] = serde_json::json!("Names"),
            "sound" => candidate["sound"] = serde_json::json!("{\"version\":999}"),
            _ => unreachable!(),
        }
        assert!(
            game.restore_snapshot_json(&candidate.to_string()).is_err(),
            "{path}"
        );
        assert_eq!(
            game.snapshot_json().unwrap(),
            checkpoint,
            "{path} rejected atomically"
        );
        assert_eq!(game.observe(), original);
    }
}

#[test]
fn fetched_named_track_order_selects_added_game_content_and_snapshot_binds_assets() {
    use crate::world::tracks;
    use romatlas_port::assets::{FileAsset, Manifest};
    let rom = Rom::open(Path::new("../input/game.rom"), crate::config::ROM_SHA256).unwrap();
    let mut overrides = MemoryAssets {
        enabled: true,
        ..Default::default()
    };
    overrides.manifests.insert(
        "resources.json".into(),
        Manifest {
            version: 1,
            entries: vec![],
            order: vec![],
        },
    );
    let added = serde_json::json!({"version":1,"projection_offset":[0,0],"projection_scale":[103,67],"terrain":vec![0;tracks::TERRAIN_BYTES],"backgrounds":[vec![0;tracks::BACKGROUND_BYTES],vec![0;tracks::BACKGROUND_BYTES]],"graphics":vec![0;tracks::GRAPHICS_BYTES],"routes":[[4226,4226,4226,4226,4226,4226,65282],[4226,4226,4226,4226,4226,4226,65282]],"headings":[128,0],"presentation":{"countdown":{"center":[91,139],"destination_word":18688},"title":{"rectangle":{"width_flags":32770,"rows":1,"palette_remap":[0,1,2,3,4,5,6,7]},"words":[8193,8194],"destination_word":18688},"banner_destination_word":18560,"hud_anchor":[7,9],"overlays":[]},"objects":{"placements":{"declared_count":1,"prefix":[{"world":[4096,8192],"height":0}]},"pickups":vec![serde_json::json!({"world":[4096,8192],"height":0});4],"dynamic":vec![serde_json::json!({"world":[0,0],"height":0});6],"composites":null,"track_three_sprites":false}});
    let mut order = vec!["native-added".into()];
    order.extend((0..16).map(|id| format!("track-{id:02}")));
    overrides.manifests.insert(
        "tracks.json".into(),
        Manifest {
            version: 1,
            entries: vec![FileAsset {
                id: "native-added".into(),
                file: "added.json".into(),
            }],
            order,
        },
    );
    overrides
        .files
        .insert("added.json".into(), serde_json::to_vec(&added).unwrap());
    let mut game = Port::load_assets(&rom, &overrides).unwrap();
    game.state.scene = Scene::RacePrepare;
    game.state.session.active_humans = 1;
    game.tick(&InputState::default());
    assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
    assert_eq!(game.observe()["progression"]["track"], "native-added");
    assert_eq!(game.observe()["progression"]["sequence_count"], 17);
    assert_eq!(game.state.scene, Scene::RaceFade);
    assert_eq!(game.state.race.world.placements.len(), 1);
    assert_eq!(game.state.race.timer.encoded()[0], 0x16);
    assert_eq!(
        &game.state.race.video.memory.vram[0x9200..0x9204],
        &[1, 32, 2, 32]
    );
    let checkpoint = game.snapshot_json().unwrap();
    let mut original = port();
    let before = original.snapshot_json().unwrap();
    assert!(original.restore_snapshot_json(&checkpoint).is_err());
    assert_eq!(original.snapshot_json().unwrap(), before);
    game.restore_snapshot_json(&checkpoint).unwrap();
}

#[test]
fn losing_human_penalty_upload_and_prompt_release_resume_on_distinct_nmi_boundaries() {
    use crate::frontend::{results, shop};
    let mut game = port();
    game.state.session.records = game.resources.results.defaults.clone();
    shop::initialize_wallets(&mut game.state.session);
    game.state.session.active_humans = 8;
    game.state.session.joined_pages = [0, 2];
    game.state.session.continuation_progress = 1;
    game.state.session.inhibit_video = 1;
    for truck in &mut game.state.race.trucks {
        truck.upgrades = crate::trucks::motion::Upgrades {
            steering: 6,
            speed: 6,
            acceleration: 6,
            suspension: 6,
        };
    }
    let route = game.selected.track.routes[usize::from(game.selected.reversed)].clone();
    game.state.postrace = Some(results::Postrace {
        route_count: route.checkpoint_count,
        route: Some(route),
        loss_route: None,
        phase: results::PostracePhase::LossPrelude,
        rank: [0, 1, 2, 3],
        awards: results::Awards {
            timer: 0,
            new_record_pending: 0,
            phase: results::AwardsFrame::Dismissed,
        },
    });
    game.state.scene = Scene::Postrace;
    game.sync_race_views();
    let before = game.state.race.video.memory.vram.clone();
    let event_begin = game.state.next_event;
    game.tick(&InputState::default());
    assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
    assert_eq!(game.state.scene, Scene::LossSetup);
    assert_eq!(game.state.session.inhibit_video, 2);
    assert_eq!(game.state.race.trucks[3].upgrades.speed, 4);
    assert_eq!(game.state.session.wallets[3], [0, 0, 1, 0, 0, 0]);
    assert_eq!(
        game.state.race.video.memory.vram, before,
        "upload waits after the penalty"
    );
    assert!(
        !game
            .state
            .events
            .iter()
            .filter(|event| event.id >= event_begin)
            .any(|event| event.kind == "sound_requested" && event.details["argument"] == 16)
    );
    game.tick(&InputState::default());
    assert_eq!(game.state.scene, Scene::LossShowWait);
    assert_eq!(game.state.session.inhibit_video, 2);
    assert_ne!(game.state.race.video.memory.vram, before);
    assert_eq!(
        game.state
            .events
            .iter()
            .filter(|event| event.id >= event_begin
                && event.kind == "sound_requested"
                && event.details["argument"] == 16)
            .count(),
        1
    );
    game.tick(&InputState::default());
    assert_eq!(game.state.scene, Scene::LossFade);
    assert_eq!(game.state.session.inhibit_video, 0);
    assert_eq!(&game.state.race.video.memory.staged_oam[..2], &[0x24, 0xae]);
    let checkpoint = game.snapshot_json().unwrap();
    let mut branch = game.clone();
    game.tick(&InputState::default());
    branch.tick(&InputState::default());
    assert_eq!(
        &game.state.race.video.memory.visible_oam[..2],
        &[0x24, 0xae]
    );
    let mut expected = Vec::new();
    branch.audio(83, 44100, &mut expected);
    game.restore_snapshot_json(&checkpoint).unwrap();
    game.tick(&InputState::default());
    let mut actual = Vec::new();
    game.audio(83, 44100, &mut actual);
    assert_eq!(actual, expected);
    assert_eq!(game.observe(), branch.observe());
}

#[test]
fn championship_default_menu_renders_and_arcade_random_still_enters_names() {
    let mut game = port();
    game.tick(&InputState::default());
    game.state.automatic[0] = crate::controls::ControllerWord(crate::controls::BUTTON_START);
    game.begin_names();
    assert_eq!(game.state.scene, Scene::ArcadeMenu);
    let mut held = InputState::default();
    held.held.insert("start".into());
    game.tick(&held);
    assert_eq!(game.state.scene, Scene::ArcadeMenu);
    let mut frame = Frame::new(256, 224).unwrap();
    game.render(&mut frame);
    assert!(
        frame
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel[..3] != [0, 0, 0])
    );
    assert!(
        frame.pixels[..256 * 4 * 32]
            .chunks_exact(4)
            .all(|pixel| pixel[..3] == [0, 0, 0])
    );
    let saved = game.snapshot_json().unwrap();
    let mut receiver = port();
    receiver.restore_snapshot_json(&saved).unwrap();
    game.tick(&InputState::default());
    let mut down = InputState::default();
    down.held.insert("down".into());
    game.tick(&down);
    game.tick(&InputState::default());
    game.tick(&held);
    assert_eq!(game.state.scene, Scene::Names);
    assert!(game.state.arcade.accepted);
    assert_eq!(game.state.arcade.order.len(), 16);
    for _ in 0..120 {
        game.tick(&InputState::default());
    }
    assert!(game.state.fault.is_none(), "{:?}", game.state.fault);
}

#[test]
fn arcade_random_assets_restore_direction_and_valid_bales_while_classic_is_unchanged() {
    let mut game = port();
    game.state.random = crate::runtime::RandomState([18, 52, 86, 120, 155]);
    game.state.arcade.accepted = true;
    game.state
        .arcade
        .start(&game.resources.sequence, &mut game.state.random);
    let mut saw_forward = false;
    let mut saw_reverse = false;
    let mut saw_changed_bales = false;
    for cursor in 0..16 {
        game.state.arcade.cursor = cursor;
        game.state.sequence_index = game.state.arcade.order[cursor];
        game.state.arcade.roll(&mut game.state.random);
        let selected = game.resources.arcade_race(&game.state).unwrap();
        saw_reverse |= selected.reversed;
        saw_forward |= !selected.reversed;
        let original = game.resources.sequence[game.state.sequence_index]
            .original
            .unwrap();
        let slots = crate::world::tracks::decode_order(
            &game.resources.rom
                [crate::world::tracks::ORDER_ROM..crate::world::tracks::ORDER_ROM + 64],
        )
        .unwrap();
        let valid: Vec<_> = slots
            .iter()
            .enumerate()
            .filter(|(_, c)| c.original_track == original)
            .flat_map(|(i, _)| {
                crate::world::objects::load_original_content(&game.resources.rom, original, i as u8)
                    .unwrap()
                    .dynamic
            })
            .collect();
        for bale in selected.objects.dynamic {
            assert!(bale.world[0] == 0 || valid.contains(&bale));
        }
        saw_changed_bales |= selected.objects.dynamic
            != game
                .resources
                .race(game.state.sequence_index, 0)
                .unwrap()
                .objects
                .dynamic;
        let saved = game.snapshot_json().unwrap();
        let mut receiver = port();
        receiver.restore_snapshot_json(&saved).unwrap();
        assert_eq!(receiver.selected.reversed, selected.reversed);
        assert_eq!(receiver.selected.objects, selected.objects);
    }
    assert!(saw_forward && saw_reverse && saw_changed_bales);
    game.state.arcade.random = false;
    let before = game.state.random;
    game.state.arcade.roll(&mut game.state.random);
    let classic = game.resources.arcade_race(&game.state).unwrap();
    let source = game
        .resources
        .race(game.state.sequence_index, game.state.session.order_slot)
        .unwrap();
    assert_eq!(classic.reversed, source.reversed);
    assert_eq!(classic.objects, source.objects);
    assert_eq!(game.state.random, before);
}

#[test]
fn arcade_random_race_runs_and_snapshot_replays_frames() {
    let mut game = port();
    game.tick(&InputState::default());
    game.state.session.signature = game.resources.frontend.signature;
    game.state.session.records = game.resources.results.defaults.clone();
    crate::frontend::shop::initialize_wallets(&mut game.state.session);
    game.state.session.active_humans = 1;
    game.state.session.joined_pages = [1, 0];
    game.state.random = crate::runtime::RandomState([18, 52, 86, 120, 155]);
    game.state.arcade.accepted = true;
    game.state
        .arcade
        .start(&game.resources.sequence, &mut game.state.random);
    game.state.arcade.roll(&mut game.state.random);
    game.state.sequence_index = game.state.arcade.order[0];
    game.state.scene = Scene::RacePrepare;
    for _ in 0..600 {
        game.tick(&InputState::default());
        if game.state.scene == Scene::Racing {
            break;
        }
    }
    assert_eq!(game.state.scene, Scene::Racing, "{:?}", game.state.fault);
    let mut receiver = port();
    receiver
        .restore_snapshot_json(&game.snapshot_json().unwrap())
        .unwrap();
    for _ in 0..60 {
        game.tick(&InputState::default());
        receiver.tick(&InputState::default());
    }
    assert!(game.state.fault.is_none());
    assert_eq!(game.observe(), receiver.observe());
}

#[test]
fn track15_bottom_boundary_crash_regression_finishes_race() {
    // Minimal racing owners extracted from the user's tick-8469 crash. Video,
    // audio, and terrain are rebuilt normally from the original assets.
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/track15-boundary.json")).unwrap();
    let mut game = port();
    game.tick(&InputState::default());
    game.state.session.signature = game.resources.frontend.signature;
    game.state.session.records = game.resources.results.defaults.clone();
    crate::frontend::shop::initialize_wallets(&mut game.state.session);
    game.state.session.active_humans = 1;
    game.state.session.joined_pages = [1, 0];
    game.state.session.order_slot = 1;
    game.state.arcade = serde_json::from_value(fixture["arcade"].clone()).unwrap();
    game.state.sequence_index = fixture["sequence_index"].as_u64().unwrap() as usize;
    game.state.random = serde_json::from_value(fixture["random"].clone()).unwrap();
    game.state.scene = Scene::RacePrepare;
    for _ in 0..600 {
        game.tick(&InputState::default());
        if game.state.scene == Scene::Racing {
            break;
        }
    }
    assert_eq!(game.state.scene, Scene::Racing);
    game.state.race.trucks = serde_json::from_value(fixture["trucks"].clone()).unwrap();
    game.state.race.progress = serde_json::from_value(fixture["progress"].clone()).unwrap();
    game.state.race.terrain_history =
        serde_json::from_value(fixture["terrain_history"].clone()).unwrap();
    game.state.random = serde_json::from_value(fixture["random"].clone()).unwrap();
    let mut input = InputState::default();
    input.held.insert("b".into());
    let mut clock = romatlas_port::audio::SampleClock::new(crate::config::RATE, 48000).unwrap();
    let mut finished = false;
    for tick in 0..12000 {
        game.tick(&input);
        let mut audio = Vec::new();
        game.audio(clock.next_tick(), 48000, &mut audio);
        assert!(
            game.state.fault.is_none(),
            "tick {tick}: {:?}",
            game.state.fault
        );
        if game.state.scene == Scene::Postrace {
            finished = true;
            break;
        }
    }
    assert!(finished, "did not finish race: {:?}", game.state.scene);
}
