use smac_core::factions::*;

#[test]
fn test_all_factions() {
    let factions = all_factions();
    assert!(!factions.is_empty());
    assert!(factions
        .iter()
        .any(|faction| faction.name == "Gaia's Stepdaughters"));
    assert!(factions.iter().any(|faction| faction.color == "Blue"));
}
