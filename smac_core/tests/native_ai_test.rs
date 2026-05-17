use smac_core::{GameState, Terrain, Unit, UnitKind};

#[test]
fn native_unit_moves_toward_player_target_on_end_turn() {
    let mut game = GameState::new_game(16, 16, 7);
    for f in game.factions.iter_mut() {
        f.base_attributes.morale = 0;
    }
    let native_owner = game.native_owner();
    let player_owner = game.player_owner();

    let native_id = game.units.len();
    let nx = 12;
    let ny = 12;
    let nidx = ny * game.width + nx;
    game.tiles[nidx].terrain = Terrain::Fungus;
    game.tiles[nidx].unit = Some(native_id);

    game.units.push(Unit {
        id: native_id,
        owner: native_owner,
        kind: UnitKind::MindWorm,
        design_index: 0,
        x: nx,
        y: ny,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    let player_id = game.units.len();
    let px = 10;
    let py = 10;
    let pidx = py * game.width + px;
    game.tiles[pidx].terrain = Terrain::Flat;
    game.tiles[pidx].unit = Some(player_id);

    game.units.push(Unit {
        id: player_id,
        owner: player_owner,
        kind: UnitKind::ScoutPatrol,
        design_index: 0,
        x: px,
        y: py,
        moves_left: 1,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });

    game.end_turn();

    let native = game.unit(native_id).expect("native should still exist");
    assert_eq!((native.x, native.y), (11, 11));
}

#[test]
fn native_life_spawns_from_fungus_on_spawn_turn() {
    let mut game = GameState::new_game(12, 12, 7);
    let native_owner = game.native_owner();

    game.units.retain(|unit| unit.owner != native_owner);
    for tile in &mut game.tiles {
        if tile.unit.is_some() {
            tile.unit = None;
        }
        tile.terrain = Terrain::Fungus;
    }
    game.turn = 5;

    let before = game
        .units
        .iter()
        .filter(|unit| unit.owner == native_owner && unit.alive)
        .count();

    game.end_turn();

    let after = game
        .units
        .iter()
        .filter(|unit| unit.owner == native_owner && unit.alive)
        .count();

    assert!(after > before);
}
