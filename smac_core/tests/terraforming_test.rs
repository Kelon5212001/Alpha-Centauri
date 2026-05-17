use smac_core::{GameState, Improvement, Terrain};

#[test]
fn condenser_nutrient_doubling() {
    let mut game = GameState::new_game(12, 12, 7);

    // Set a tile to Flat (2 nutrients)
    game.tiles[5 * game.width + 5].terrain = Terrain::Flat;
    game.tiles[5 * game.width + 5].improvement = None;

    let base_yields = game.tile_total_yields(5, 5);
    assert_eq!(base_yields.nutrients, 2);

    // Add Condenser (+2 nutrients additive in Improvement::yields, then doubled)
    // Terrain: 2, Improvement: 2 -> Total 4. Doubled -> 8.
    game.tiles[5 * game.width + 5].improvement = Some(Improvement::Condenser);

    let boosted_yields = game.tile_total_yields(5, 5);
    assert_eq!(boosted_yields.nutrients, 8);
}

#[test]
fn echelon_mirror_solar_adjacency() {
    let mut game = GameState::new_game(12, 12, 7);

    // 1. Solar collector alone
    game.tiles[5 * game.width + 5].terrain = Terrain::Flat; // 1 energy
    game.tiles[5 * game.width + 5].improvement = Some(Improvement::Solar); // +2 energy

    let base_yields = game.tile_total_yields(5, 5);
    assert_eq!(base_yields.energy, 3);

    // 2. Add adjacent mirror
    game.tiles[5 * game.width + 4].improvement = Some(Improvement::EchelonMirror);

    let boosted_yields = game.tile_total_yields(5, 5);
    assert_eq!(boosted_yields.energy, 4); // 3 base + 1 from 1 adjacent mirror

    // 3. Add another adjacent mirror
    game.tiles[6 * game.width + 5].improvement = Some(Improvement::EchelonMirror);

    let double_boosted_yields = game.tile_total_yields(5, 5);
    assert_eq!(double_boosted_yields.energy, 5); // 3 base + 2 from 2 adjacent mirrors
}
