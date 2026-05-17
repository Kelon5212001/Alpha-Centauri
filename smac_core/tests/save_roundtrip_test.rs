use smac_core::{
    current_save_slot_label, filtered_sorted_save_slots, matches_save_filters, save_browser_counts,
    save_browser_counts_text, save_browser_display_state, save_filter_label, save_slot_label,
    save_sort_label, set_save_sort, sort_save_slots, Base, GameState, GameStateSnapshot,
    GovernorMode, ProductionItem, SaveBrowserQuery, SaveFilterCategory, SaveSlotCategory,
    SaveSlotListing, SaveSlotMetadata, SaveSortColumn, GAME_STATE_SNAPSHOT_VERSION,
};
use std::fs;

fn empty_tiles(width: usize, height: usize) -> Vec<smac_core::Tile> {
    let mut tiles = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            tiles.push(smac_core::Tile {
                x,
                y,
                terrain: smac_core::Terrain::Flat,
                elevation: 0,
                moisture: 50,
                unit: None,
                base: None,
                pod: false,
                improvement: None,
                explored_by_owner: Default::default(),
                visible_by_owner: Default::default(),
            });
        }
    }
    tiles
}

#[test]
fn snapshot_json_roundtrip_preserves_core_state() {
    let game = GameState::new_game(16, 16, 42);

    let mut snapshot = GameStateSnapshot::from(&game);
    snapshot.save_name = Some("Roundtrip Test".to_string());
    let json = snapshot
        .to_json_pretty()
        .expect("snapshot should serialize to json");
    let restored = GameStateSnapshot::from_json(&json)
        .expect("snapshot json should deserialize")
        .into_game_state();

    assert_eq!(restored.width, game.width);
    assert_eq!(restored.height, game.height);
    assert_eq!(restored.seed, game.seed);
    assert_eq!(restored.turn, game.turn);
    assert_eq!(snapshot.version, GAME_STATE_SNAPSHOT_VERSION);
    assert_eq!(restored.log.len(), game.log.len());
    assert_eq!(restored.tiles.len(), game.tiles.len());
    assert_eq!(restored.units.len(), game.units.len());
    assert_eq!(restored.bases.len(), game.bases.len());
    assert_eq!(restored.factions.len(), game.factions.len());
    assert_eq!(GameStateSnapshot::from(&restored).save_name, None);
}

#[test]
fn snapshot_roundtrip_preserves_convoy_routes() {
    let mut game = GameState::new_game(16, 16, 42);
    game.units.clear();
    game.bases.clear();
    for tile in &mut game.tiles {
        tile.unit = None;
        tile.base = None;
    }

    game.bases.push(Base {
        id: 0,
        owner: game.player_owner(),
        name: "Alpha".to_string(),
        x: 4,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 4].base = Some(0);
    game.bases.push(Base {
        id: 1,
        owner: game.player_owner(),
        name: "Beta".to_string(),
        x: 8,
        y: 4,
        population: 2,
        nutrients_stock: 0,
        minerals_stock: 0,
        production: ProductionItem::ScoutPatrol,
        production_queue: Vec::new(),
        facilities: Vec::new(),
        governor_mode: GovernorMode::Off,
    });
    game.tiles[4 * game.width + 8].base = Some(1);
    game.add_convoy_route(0, 1).expect("route should be added");

    let mut snapshot = GameStateSnapshot::from(&game);
    snapshot.save_name = Some("Convoy Route Test".to_string());
    let json = snapshot
        .to_json_pretty()
        .expect("snapshot should serialize");
    let restored = GameStateSnapshot::from_json(&json)
        .expect("snapshot should deserialize")
        .into_game_state();

    assert_eq!(restored.convoy_routes.len(), 1);
    assert_eq!(restored.base_trade_links(0), 1);
}

#[test]
fn snapshot_can_save_and_load_from_file() {
    let game = GameState::new_game(12, 12, 99);
    let mut snapshot = GameStateSnapshot::from(&game);
    snapshot.save_name = Some("Metadata Test".to_string());
    let path = std::env::temp_dir().join(format!("smac_snapshot_{}_test.json", std::process::id()));

    snapshot
        .save_to_path(&path)
        .expect("snapshot should save to a file");
    let restored = GameStateSnapshot::load_from_path(&path)
        .expect("snapshot should load from a file")
        .into_game_state();

    assert_eq!(restored.width, game.width);
    assert_eq!(restored.height, game.height);
    assert_eq!(restored.turn, game.turn);

    let metadata = SaveSlotMetadata::load_from_path(&path).expect("metadata should load");
    assert_eq!(metadata.save_name, "Metadata Test");
    assert_eq!(metadata.saved_turn, game.turn);
    assert_eq!(metadata.category, Some(SaveSlotCategory::Manual));

    let _ = fs::remove_file(path);
}

#[test]
fn snapshot_rejects_unsupported_version() {
    let game = GameState::new_game(12, 12, 99);
    let mut snapshot = GameStateSnapshot::from(&game);
    snapshot.version = GAME_STATE_SNAPSHOT_VERSION + 1;

    let json = snapshot
        .to_json_pretty()
        .expect("snapshot should serialize even with altered version");

    let err = GameStateSnapshot::from_json(&json).expect_err("unsupported version should fail");
    assert!(err.to_string().contains("unsupported snapshot version"));
}

#[test]
fn snapshot_migrates_legacy_versionless_json() {
    let width = 8;
    let height = 8;
    let legacy_v0 = serde_json::json!({
        "width": width,
        "height": height,
        "seed": 5,
        "turn": 3,
        "tiles": empty_tiles(width, height),
        "units": [],
        "bases": [],
        "factions": [],
        "log": [],
        "game_over": null
    });

    let json = serde_json::to_string_pretty(&legacy_v0).expect("legacy json should serialize");
    let migrated = GameStateSnapshot::from_json(&json).expect("legacy snapshot should migrate");

    assert_eq!(migrated.version, GAME_STATE_SNAPSHOT_VERSION);
    assert_eq!(migrated.save_name.as_deref(), Some("Imported Legacy Save"));
    assert_eq!(migrated.width, width);
    assert_eq!(migrated.height, height);
}

#[test]
fn snapshot_migrates_v1_to_v2() {
    let legacy_v1 = serde_json::json!({
        "version": 1,
        "width": 8,
        "height": 8,
        "seed": 5,
        "turn": 3,
        "tiles": empty_tiles(8, 8),
        "units": [],
        "bases": [],
        "factions": [],
        "log": [],
        "game_over": null
    });

    let migrated = GameStateSnapshot::from_json(
        &serde_json::to_string_pretty(&legacy_v1).expect("legacy v1 json should serialize"),
    )
    .expect("v1 snapshot should migrate");

    assert_eq!(migrated.version, GAME_STATE_SNAPSHOT_VERSION);
    assert_eq!(migrated.save_name.as_deref(), Some("Migrated Save"));
    assert_eq!(migrated.saved_turn, 3);
}

#[test]
fn snapshot_rejects_unrecoverable_owner_corruption() {
    let game = GameState::new_game(10, 10, 77);
    let mut snapshot = GameStateSnapshot::from(&game);

    if let Some(unit) = snapshot.units.iter().find(|unit| unit.alive).cloned() {
        let corrupt_unit = snapshot
            .units
            .iter_mut()
            .find(|candidate| candidate.id == unit.id)
            .expect("unit should exist in snapshot");
        corrupt_unit.owner = 999;
    }

    let json = snapshot
        .to_json_pretty()
        .expect("corrupt snapshot should still serialize");

    let err = GameStateSnapshot::from_json(&json).expect_err("corrupt snapshot should fail");
    assert!(err.to_string().contains("invalid owner"));
}

#[test]
fn snapshot_repairs_minor_tile_coordinate_and_link_drift() {
    let game = GameState::new_game(10, 10, 77);
    let mut snapshot = GameStateSnapshot::from(&game);

    snapshot.tiles[0].x = 99;
    snapshot.tiles[0].y = 99;
    if let Some(unit) = snapshot.units.iter().find(|unit| unit.alive).cloned() {
        let idx = unit.y * snapshot.width + unit.x;
        snapshot.tiles[idx].unit = None;
    }
    snapshot.save_name = Some(" ".to_string());
    snapshot.saved_turn = -5;

    let json = snapshot
        .to_json_pretty()
        .expect("drifted snapshot should still serialize");
    let repaired = GameStateSnapshot::from_json(&json).expect("repairable snapshot should load");

    assert_eq!(repaired.tiles[0].x, 0);
    assert_eq!(repaired.tiles[0].y, 0);
    assert_eq!(repaired.saved_turn, repaired.turn);
    assert_eq!(repaired.save_name.as_deref(), Some("Recovered Save"));
    assert!(!repaired.recovery_notes.is_empty());
}

#[test]
fn snapshot_current_version_loads_without_optional_recovery_notes() {
    let game = GameState::new_game(10, 10, 77);
    let mut snapshot = GameStateSnapshot::from(&game);
    snapshot.save_name = Some("Compat Test".to_string());
    let mut value = serde_json::to_value(snapshot).expect("snapshot should serialize");
    value
        .as_object_mut()
        .expect("snapshot should be object")
        .remove("recovery_notes");

    let json = serde_json::to_string_pretty(&value).expect("json should serialize");
    let restored = GameStateSnapshot::from_json(&json).expect("snapshot should load");

    assert_eq!(restored.width, game.width);
    assert_eq!(restored.height, game.height);
    assert_eq!(restored.turn, game.turn);
}

#[test]
fn snapshot_current_version_loads_without_optional_save_name() {
    let game = GameState::new_game(10, 10, 77);
    let snapshot = GameStateSnapshot::from(&game);
    let mut value = serde_json::to_value(snapshot).expect("snapshot should serialize");
    value
        .as_object_mut()
        .expect("snapshot should be object")
        .remove("save_name");

    let json = serde_json::to_string_pretty(&value).expect("json should serialize");
    let restored = GameStateSnapshot::from_json(&json).expect("snapshot should load");

    assert_eq!(restored.width, game.width);
    assert_eq!(restored.height, game.height);
    assert_eq!(restored.turn, game.turn);
}

#[test]
fn save_slot_metadata_sidecar_loads_without_optional_recovery_count() {
    let path = std::env::temp_dir().join(format!("smac_metadata_{}_test.json", std::process::id()));
    let metadata_path = std::env::temp_dir().join(format!(
        "smac_metadata_{}_test.json.meta",
        std::process::id()
    ));
    let metadata_json = serde_json::json!({
        "save_name": "Sidecar Compat",
        "saved_turn": 9
    });

    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata_json).expect("metadata json should serialize"),
    )
    .expect("metadata sidecar should write");

    let metadata = SaveSlotMetadata::load_from_path(&path).expect("metadata should load");
    assert_eq!(metadata.save_name, "Sidecar Compat");
    assert_eq!(metadata.saved_turn, 9);
    assert_eq!(metadata.recovery_note_count, 0);
    assert_eq!(metadata.notes, "");
    assert_eq!(metadata.category, None);

    let _ = fs::remove_file(metadata_path);
}

#[test]
fn save_slot_metadata_sidecar_ignores_unknown_fields() {
    let path = std::env::temp_dir().join(format!(
        "smac_metadata_unknown_{}_test.json",
        std::process::id()
    ));
    let metadata_path = std::env::temp_dir().join(format!(
        "smac_metadata_unknown_{}_test.json.meta",
        std::process::id()
    ));
    let metadata_json = serde_json::json!({
        "save_name": "Unknown Field Compat",
        "saved_turn": 12,
        "recovery_note_count": 1,
        "future_flag": true
    });

    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata_json).expect("metadata json should serialize"),
    )
    .expect("metadata sidecar should write");

    let metadata = SaveSlotMetadata::load_from_path(&path).expect("metadata should load");
    assert_eq!(metadata.save_name, "Unknown Field Compat");
    assert_eq!(metadata.saved_turn, 12);
    assert_eq!(metadata.recovery_note_count, 1);

    let _ = fs::remove_file(metadata_path);
}

#[test]
fn save_slot_metadata_sidecar_loads_without_optional_timestamp() {
    let path = std::env::temp_dir().join(format!(
        "smac_metadata_time_{}_test.json",
        std::process::id()
    ));
    let metadata_path = std::env::temp_dir().join(format!(
        "smac_metadata_time_{}_test.json.meta",
        std::process::id()
    ));
    let metadata_json = serde_json::json!({
        "save_name": "Timestamp Compat",
        "saved_turn": 15,
        "recovery_note_count": 2
    });

    fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata_json).expect("metadata json should serialize"),
    )
    .expect("metadata sidecar should write");

    let metadata = SaveSlotMetadata::load_from_path(&path).expect("metadata should load");
    assert_eq!(metadata.save_name, "Timestamp Compat");
    assert_eq!(metadata.saved_turn, 15);
    assert_eq!(metadata.recovery_note_count, 2);
    assert_eq!(metadata.notes, "");
    assert_eq!(metadata.category, None);

    let _ = fs::remove_file(metadata_path);
}

#[test]
fn save_slot_discovery_lists_existing_slots_and_next_empty_slot() {
    let unique = std::process::id();
    let dir = std::env::temp_dir().join(format!("smac_slot_listing_{unique}"));
    fs::create_dir_all(&dir).expect("listing directory should exist");

    fs::write(
        dir.join("slot_2.json.meta"),
        serde_json::to_string_pretty(&serde_json::json!({
            "save_name": "Second Slot",
            "saved_turn": 8,
            "recovery_note_count": 0
        }))
        .expect("metadata should serialize"),
    )
    .expect("slot 2 metadata should write");
    fs::write(
        dir.join("slot_4.json.meta"),
        serde_json::to_string_pretty(&serde_json::json!({
            "save_name": "Fourth Slot",
            "saved_turn": 12,
            "recovery_note_count": 1
        }))
        .expect("metadata should serialize"),
    )
    .expect("slot 4 metadata should write");

    let listings = SaveSlotMetadata::discover_slots(&dir);
    let ids: Vec<&str> = listings.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(ids, vec!["slot_2", "slot_4", "slot_5"]);
    assert_eq!(listings[0].category, SaveSlotCategory::Manual);
    assert_eq!(
        listings[0]
            .metadata
            .as_ref()
            .expect("slot 2 metadata should exist")
            .save_name,
        "Second Slot"
    );
    assert!(listings[2].metadata.is_none());

    let _ = fs::remove_file(dir.join("slot_2.json.meta"));
    let _ = fs::remove_file(dir.join("slot_4.json.meta"));
    let _ = fs::remove_dir(&dir);
}

#[test]
fn save_slot_discovery_classifies_autosave_manual_imported_and_empty() {
    let unique = std::process::id();
    let dir = std::env::temp_dir().join(format!("smac_slot_categories_{unique}"));
    fs::create_dir_all(&dir).expect("category directory should exist");

    for (name, title) in [
        ("autosave_1.json.meta", "Auto"),
        ("slot_3.json.meta", "Manual"),
        ("custom_campaign.json.meta", "Imported"),
    ] {
        fs::write(
            dir.join(name),
            serde_json::to_string_pretty(&serde_json::json!({
                "save_name": title,
                "saved_turn": 6,
                "recovery_note_count": 0,
                "notes": format!("{title} notes")
            }))
            .expect("metadata should serialize"),
        )
        .expect("metadata sidecar should write");
    }

    let listings = SaveSlotMetadata::discover_slots(&dir);
    let autosave = listings
        .iter()
        .find(|entry| entry.id == "autosave_1")
        .expect("autosave listing should exist");
    let manual = listings
        .iter()
        .find(|entry| entry.id == "slot_3")
        .expect("manual listing should exist");
    let imported = listings
        .iter()
        .find(|entry| entry.id == "custom_campaign")
        .expect("imported listing should exist");
    let empty = listings
        .iter()
        .find(|entry| entry.metadata.is_none())
        .expect("next empty listing should exist");

    assert_eq!(autosave.category, SaveSlotCategory::Autosave);
    assert_eq!(manual.category, SaveSlotCategory::Manual);
    assert_eq!(imported.category, SaveSlotCategory::Imported);
    assert_eq!(empty.category, SaveSlotCategory::Empty);
    assert_eq!(
        imported
            .metadata
            .as_ref()
            .expect("imported metadata should exist")
            .notes,
        "Imported notes"
    );

    let _ = fs::remove_file(dir.join("autosave_1.json.meta"));
    let _ = fs::remove_file(dir.join("slot_3.json.meta"));
    let _ = fs::remove_file(dir.join("custom_campaign.json.meta"));
    let _ = fs::remove_dir(&dir);
}

#[test]
fn save_slot_listing_can_rename_duplicate_and_delete() {
    let unique = std::process::id();
    let dir = std::env::temp_dir().join(format!("smac_slot_ops_{unique}"));
    fs::create_dir_all(&dir).expect("ops directory should exist");

    let game = GameState::new_game(12, 12, 99);
    let mut snapshot = GameStateSnapshot::from(&game);
    snapshot.save_name = Some("Ops Save".to_string());
    let source_path = dir.join("custom_alpha.json");
    snapshot
        .save_to_path(&source_path)
        .expect("source snapshot should save");

    let listing = SaveSlotMetadata::discover_slots(&dir)
        .into_iter()
        .find(|entry| entry.id == "custom_alpha")
        .expect("source listing should exist");

    let renamed = listing
        .rename_to("custom_beta")
        .expect("rename should succeed");
    assert_eq!(renamed.id, "custom_beta");
    assert!(dir.join("custom_beta.json").exists());
    assert!(dir.join("custom_beta.json.meta").exists());

    let duplicated = renamed
        .duplicate_to("custom_gamma")
        .expect("duplicate should succeed");
    assert_eq!(duplicated.id, "custom_gamma");
    assert!(dir.join("custom_gamma.json").exists());
    assert!(dir.join("custom_gamma.json.meta").exists());

    duplicated.delete().expect("delete should succeed");
    assert!(!dir.join("custom_gamma.json").exists());
    assert!(!dir.join("custom_gamma.json.meta").exists());

    let _ = fs::remove_file(dir.join("custom_beta.json"));
    let _ = fs::remove_file(dir.join("custom_beta.json.meta"));
    let _ = fs::remove_dir(&dir);
}

#[test]
fn save_slot_listing_can_import_export_and_overwrite() {
    let unique = std::process::id();
    let source_dir = std::env::temp_dir().join(format!("smac_slot_import_source_{unique}"));
    let target_dir = std::env::temp_dir().join(format!("smac_slot_import_target_{unique}"));
    let export_dir = std::env::temp_dir().join(format!("smac_slot_export_target_{unique}"));
    fs::create_dir_all(&source_dir).expect("source directory should exist");
    fs::create_dir_all(&target_dir).expect("target directory should exist");
    fs::create_dir_all(&export_dir).expect("export directory should exist");

    let game = GameState::new_game(12, 12, 99);
    let mut snapshot = GameStateSnapshot::from(&game);
    snapshot.save_name = Some("Imported Save".to_string());
    let source_path = source_dir.join("external_save.json");
    snapshot
        .save_to_path(&source_path)
        .expect("source snapshot should save");
    SaveSlotMetadata {
        save_name: "Imported Save".to_string(),
        saved_turn: game.turn,
        recovery_note_count: 0,
        last_updated_unix: None,
        notes: "Imported from external slot".to_string(),
        category: Some(SaveSlotCategory::Imported),
        auto_recovery_base_ids: Vec::new(),
        auto_defense_base_ids: Vec::new(),
        auto_economy_base_ids: Vec::new(),
    }
    .save_to_path(source_dir.join("external_save.json.meta"))
    .expect("source metadata should save");

    let imported = smac_core::SaveSlotListing::import_snapshot(
        &source_path,
        &target_dir,
        "custom_import",
        false,
    )
    .expect("import should succeed");
    assert_eq!(imported.id, "custom_import");
    assert!(target_dir.join("custom_import.json").exists());
    assert!(target_dir.join("custom_import.json.meta").exists());
    let imported_metadata = SaveSlotMetadata::load_from_path(target_dir.join("custom_import.json"))
        .expect("imported metadata should load");
    assert_eq!(imported_metadata.category, Some(SaveSlotCategory::Imported));
    assert_eq!(imported_metadata.notes, "Imported from external slot");

    let conflicting = GameStateSnapshot::from(&game);
    conflicting
        .save_to_path(target_dir.join("custom_conflict.json"))
        .expect("conflict snapshot should save");
    let conflict_listing = SaveSlotMetadata::discover_slots(&target_dir)
        .into_iter()
        .find(|entry| entry.id == "custom_import")
        .expect("imported listing should exist");
    assert!(conflict_listing.duplicate_to("custom_conflict").is_err());
    conflict_listing
        .duplicate_to_with_options("custom_conflict", true)
        .expect("overwrite duplicate should succeed");

    let export_path = export_dir.join("exported_save.json");
    conflict_listing
        .export_snapshot(&export_path, false)
        .expect("export should succeed");
    assert!(export_path.exists());
    assert!(export_dir.join("exported_save.json.meta").exists());
    let exported_metadata =
        SaveSlotMetadata::load_from_path(&export_path).expect("exported metadata should load");
    assert_eq!(exported_metadata.category, Some(SaveSlotCategory::Imported));
    assert_eq!(exported_metadata.notes, "Imported from external slot");

    let _ = fs::remove_file(source_dir.join("external_save.json"));
    let _ = fs::remove_file(source_dir.join("external_save.json.meta"));
    let _ = fs::remove_dir(&source_dir);
    let _ = fs::remove_file(target_dir.join("custom_import.json"));
    let _ = fs::remove_file(target_dir.join("custom_import.json.meta"));
    let _ = fs::remove_file(target_dir.join("custom_conflict.json"));
    let _ = fs::remove_file(target_dir.join("custom_conflict.json.meta"));
    let _ = fs::remove_dir(&target_dir);
    let _ = fs::remove_file(export_dir.join("exported_save.json"));
    let _ = fs::remove_file(export_dir.join("exported_save.json.meta"));
    let _ = fs::remove_dir(&export_dir);
}

#[test]
fn save_browser_query_helpers_filter_sort_and_count_entries() {
    let entries = vec![
        SaveSlotListing {
            id: "autosave_latest".to_string(),
            snapshot_path: "saves/autosave_latest.json".into(),
            metadata_path: "saves/autosave_latest.json.meta".into(),
            metadata: Some(SaveSlotMetadata {
                save_name: "Auto".to_string(),
                saved_turn: 8,
                recovery_note_count: 0,
                last_updated_unix: Some(10),
                notes: String::new(),
                category: Some(SaveSlotCategory::Autosave),
                auto_recovery_base_ids: Vec::new(),
                auto_defense_base_ids: Vec::new(),
                auto_economy_base_ids: Vec::new(),
            }),
            category: SaveSlotCategory::Autosave,
        },
        SaveSlotListing {
            id: "slot_2".to_string(),
            snapshot_path: "saves/slot_2.json".into(),
            metadata_path: "saves/slot_2.json.meta".into(),
            metadata: Some(SaveSlotMetadata {
                save_name: "Manual Save".to_string(),
                saved_turn: 12,
                recovery_note_count: 2,
                last_updated_unix: Some(20),
                notes: "notes".to_string(),
                category: Some(SaveSlotCategory::Manual),
                auto_recovery_base_ids: Vec::new(),
                auto_defense_base_ids: Vec::new(),
                auto_economy_base_ids: Vec::new(),
            }),
            category: SaveSlotCategory::Manual,
        },
        SaveSlotListing {
            id: "imported_save".to_string(),
            snapshot_path: "saves/imported_save.json".into(),
            metadata_path: "saves/imported_save.json.meta".into(),
            metadata: None,
            category: SaveSlotCategory::Empty,
        },
    ];

    let query = SaveBrowserQuery {
        filter_text: "manual".to_string(),
        filter_category: SaveFilterCategory::All,
        recovered_only: false,
        populated_only: true,
        sort_column: SaveSortColumn::Updated,
        sort_descending: true,
    };

    assert!(matches_save_filters(&entries[1], &query));
    assert!(!matches_save_filters(&entries[0], &query));

    let counts = save_browser_counts(&entries, &query);
    assert_eq!(counts.total, 3);
    assert_eq!(counts.filtered, 1);
    assert_eq!(counts.autosave_count, 1);
    assert_eq!(counts.manual_count, 1);
    assert_eq!(counts.empty_count, 1);
    assert_eq!(
        save_browser_counts_text(counts),
        "Showing 1/3 saves. Autosaves: 1  Manual: 1  Imported: 0  Empty: 1"
    );

    let mut sorted = entries.clone();
    sort_save_slots(&mut sorted, SaveSortColumn::Turn, true);
    assert_eq!(sorted[0].id, "slot_2");
    let filtered_sorted = filtered_sorted_save_slots(&entries, &query);
    assert_eq!(filtered_sorted.len(), 1);
    assert_eq!(filtered_sorted[0].id, "slot_2");
    assert!(save_slot_label(&entries[1]).contains("Manual Save"));
    assert_eq!(
        current_save_slot_label(&entries, "slot_2"),
        save_slot_label(&entries[1])
    );
    let display = save_browser_display_state(&entries, "slot_2", &query);
    assert_eq!(display.current_slot_label, save_slot_label(&entries[1]));
    assert_eq!(
        display.counts_text,
        "Showing 1/3 saves. Autosaves: 1  Manual: 1  Imported: 0  Empty: 1"
    );
    assert_eq!(display.rows.len(), 1);
    assert_eq!(display.rows[0].id, "slot_2");
    assert!(display.rows[0].selected);
    assert!(display
        .sort_buttons
        .iter()
        .any(|button| button.column == SaveSortColumn::Updated
            && button.label_text == "Updated v"
            && button.selected));
    assert_eq!(save_sort_label(SaveSortColumn::Recovery), "Recovery");
    assert_eq!(save_filter_label(SaveFilterCategory::Autosave), "Autosaves");
    assert_eq!(
        set_save_sort(SaveSortColumn::Updated, true, SaveSortColumn::Updated),
        (SaveSortColumn::Updated, false)
    );
}
