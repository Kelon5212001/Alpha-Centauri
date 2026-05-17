use smac_core::{Economics, FutureSociety, GameState, Politics, Values};

#[test]
fn social_engineering_modifiers_apply_correctly() {
    let mut game = GameState::new_game(16, 16, 12345);
    let owner = game.player_owner();
    game.faction_mut(owner).unwrap().name = "Test Faction".to_string();

    // 1. Initial State (Frontier, Simple, Survival, None)
    {
        let attrs = game.faction(owner).unwrap().effective_attributes();
        assert_eq!(attrs.support, 0); // Gaia's Stepdaughters has 0 base support
        assert_eq!(attrs.efficiency, 2); // Gaia's Stepdaughters has +2 Efficiency
    }

    // 2. Change Politics to Police
    game.choose_social_engineering(owner, Some(Politics::Police), None, None, None)
        .unwrap();
    {
        let attrs = game.faction(owner).unwrap().effective_attributes();
        assert_eq!(attrs.support, 2); // Police +2
        assert_eq!(attrs.police, 2); // Police +2
        assert_eq!(attrs.efficiency, 0); // Police -2, Gaia +2 = 0
    }

    // 3. Change Economics to Free Market
    game.choose_social_engineering(owner, None, Some(Economics::FreeMarket), None, None)
        .unwrap();
    {
        let attrs = game.faction(owner).unwrap().effective_attributes();
        assert_eq!(attrs.economy, 2); // Free Market +2
        assert_eq!(attrs.efficiency, 2); // Free Market +2, Police -2, Gaia +2 = 2
        assert_eq!(attrs.planet, -1); // Free Market -2, Gaia +1 = -1
    }

    // 4. Change Values to Wealth
    game.choose_social_engineering(owner, None, None, Some(Values::Wealth), None)
        .unwrap();
    {
        let attrs = game.faction(owner).unwrap().effective_attributes();
        assert_eq!(attrs.economy, 3); // Wealth +1, Free Market +2 = 3
        assert_eq!(attrs.industry, 1); // Wealth +1
        assert_eq!(attrs.morale, -2); // Wealth -1, Gaia -1 = -2
    }

    // 5. Change Future to Cybernetic
    game.choose_social_engineering(owner, None, None, None, Some(FutureSociety::Cybernetic))
        .unwrap();
    {
        let attrs = game.faction(owner).unwrap().effective_attributes();
        assert_eq!(attrs.efficiency, 4); // Cybernetic +2, Free Market +2, Police -2, Gaia +2 = 4
        assert_eq!(attrs.research, 2); // Cybernetic +2
        assert_eq!(attrs.planet, 0); // Cybernetic +1, Free Market -2, Gaia +1 = 0
    }
}
