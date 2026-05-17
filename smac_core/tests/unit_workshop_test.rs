use smac_core::{Armor, Chassis, GameState, UnitDesign, UnitKind, Weapon};

#[test]
fn unit_workshop_custom_design_combat() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    let ai_owner = game.ai_owner();

    // 1. Create a "Super Tank" custom design
    let super_tank = UnitDesign {
        name: "Super Tank".to_string(),
        chassis: Chassis::Hovertank,
        weapon: Weapon::ResonanceLaser(10), // Very high attack
        armor: Armor::SynthMetal(1),
        cost: 100,
        abilities: Vec::new(),
    };

    // 2. Add design to faction
    let design_index = {
        let faction = game.faction_mut(owner).unwrap();
        faction.unit_designs.push(super_tank);
        faction.unit_designs.len() - 1
    };

    // 3. Spawn the custom unit
    // We'll manually spawn it since we haven't updated the "Set Production" action yet
    // to handle arbitrary design indices (that's Phase 2).
    let unit_id = game.units.len();
    game.units.push(smac_core::Unit {
        id: unit_id,
        owner,
        kind: UnitKind::Speeder, // We still use a kind for move type baseline for now
        design_index,
        x: 5,
        y: 5,
        moves_left: 2,
        hp: 10,
        experience: 0,
        alive: true,
        cargo_unit_ids: Vec::new(),
        activity: smac_core::UnitActivity::None,
    });
    game.tiles[5 * game.width + 5].unit = Some(unit_id);

    // 4. Verify stats are pulled from design
    assert_eq!(game.unit_attack_strength(unit_id), 10);
    let summary = game.unit_panel_summary(unit_id).unwrap();
    assert!(summary.unit_name.contains("Super Tank"));

    // 5. Combat Test
    // Spawn a weak defender
    let _defender_id = game.spawn_unit(ai_owner, UnitKind::ScoutPatrol, 5, 6);

    // Attack!
    game.move_unit_to(unit_id, 5, 6).unwrap();

    // Log should show the high attack score
    assert!(game.log.iter().any(|msg| msg.message.contains("atk: 10")
        || msg.message.contains("atk: 11")
        || msg.message.contains("atk: 12")));
}
