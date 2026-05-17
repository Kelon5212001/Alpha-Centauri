use smac_core::units::UnitPrototype;
use smac_core::TechnologyTree;
use std::collections::HashSet;

fn main() {
    let tree = TechnologyTree::new();
    println!("== Starting techs ==");
    for t in tree.get_available_technologies(&HashSet::new()) {
        println!("• {}", t.name);
    }

    let scout = UnitPrototype::scout_patrol();
    println!(
        "\nScout patrol -> A{} / D{} / cost {}",
        scout.attack_strength(),
        scout.defense_strength(),
        scout.cost
    );
}
