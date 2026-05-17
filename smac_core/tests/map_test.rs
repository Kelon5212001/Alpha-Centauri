use smac_core::map::{Map, TileType};

#[test]
fn test_map_generation_counts_land_and_ocean() {
    let map = Map::generate(32, 16, 12345);

    let mut land = 0;
    let mut ocean = 0;
    for row in &map.tiles {
        for tile in row {
            if tile.tile_type == TileType::Land {
                land += 1;
            }
            if tile.tile_type == TileType::Ocean {
                ocean += 1;
            }
        }
    }

    // Should have both land and ocean tiles present
    assert!(land > 0, "No land tiles found!");
    assert!(ocean > 0, "No ocean tiles found!");
}
