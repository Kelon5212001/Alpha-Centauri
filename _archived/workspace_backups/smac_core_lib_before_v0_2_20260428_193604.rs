pub const PLAYER_ID: usize = 0;
pub const AI_ID: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Ocean,
    Flat,
    Rolling,
    Rocky,
    Fungus,
}

impl Terrain {
    pub fn name(self) -> &'static str {
        match self {
            Terrain::Ocean => "Ocean",
            Terrain::Flat => "Flat",
            Terrain::Rolling => "Rolling",
            Terrain::Rocky => "Rocky",
            Terrain::Fungus => "Xenofungus",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Terrain::Ocean => "~",
            Terrain::Flat => ".",
            Terrain::Rolling => ",",
            Terrain::Rocky => "^",
            Terrain::Fungus => "F",
        }
    }

    pub fn yields(self) -> Yields {
        match self {
            Terrain::Ocean => Yields {
                nutrients: 1,
                minerals: 0,
                energy: 2,
            },
            Terrain::Flat => Yields {
                nutrients: 2,
                minerals: 1,
                energy: 1,
            },
            Terrain::Rolling => Yields {
                nutrients: 1,
                minerals: 2,
                energy: 1,
            },
            Terrain::Rocky => Yields {
                nutrients: 0,
                minerals: 3,
                energy: 0,
            },
            Terrain::Fungus => Yields {
                nutrients: 1,
                minerals: 1,
                energy: 0,
            },
        }
    }

    pub fn is_land(self) -> bool {
        self != Terrain::Ocean
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Yields {
    pub nutrients: i32,
    pub minerals: i32,
    pub energy: i32,
}

impl Yields {
    pub fn add(self, other: Yields) -> Yields {
        Yields {
            nutrients: self.nutrients + other.nutrients,
            minerals: self.minerals + other.minerals,
            energy: self.energy + other.energy,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub terrain: Terrain,
    pub elevation: i32,
    pub moisture: i32,
    pub unit: Option<usize>,
    pub base: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitKind {
    ColonyPod,
    ScoutPatrol,
}

impl UnitKind {
    pub fn name(self) -> &'static str {
        match self {
            UnitKind::ColonyPod => "Colony Pod",
            UnitKind::ScoutPatrol => "Scout Patrol",
        }
    }

    pub fn max_moves(self) -> i32 {
        1
    }

    pub fn attack(self) -> i32 {
        match self {
            UnitKind::ColonyPod => 0,
            UnitKind::ScoutPatrol => 2,
        }
    }

    pub fn defense(self) -> i32 {
        match self {
            UnitKind::ColonyPod => 1,
            UnitKind::ScoutPatrol => 2,
        }
    }

    pub fn can_found_base(self) -> bool {
        self == UnitKind::ColonyPod
    }
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: usize,
    pub owner: usize,
    pub kind: UnitKind,
    pub x: usize,
    pub y: usize,
    pub moves_left: i32,
    pub hp: i32,
    pub alive: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum ProductionItem {
    ScoutPatrol,
    ColonyPod,
}

impl ProductionItem {
    pub fn name(self) -> &'static str {
        match self {
            ProductionItem::ScoutPatrol => "Scout Patrol",
            ProductionItem::ColonyPod => "Colony Pod",
        }
    }

    pub fn cost(self) -> i32 {
        match self {
            ProductionItem::ScoutPatrol => 12,
            ProductionItem::ColonyPod => 24,
        }
    }

    pub fn unit_kind(self) -> UnitKind {
        match self {
            ProductionItem::ScoutPatrol => UnitKind::ScoutPatrol,
            ProductionItem::ColonyPod => UnitKind::ColonyPod,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Base {
    pub id: usize,
    pub owner: usize,
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub population: i32,
    pub nutrients_stock: i32,
    pub minerals_stock: i32,
    pub production: ProductionItem,
}

#[derive(Clone, Debug)]
pub struct Faction {
    pub id: usize,
    pub name: String,
    pub energy: i32,
    pub research: i32,
    pub techs_discovered: i32,
    pub is_ai: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOver {
    PlayerWon,
    PlayerLost,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub width: usize,
    pub height: usize,
    pub seed: u32,
    pub turn: i32,
    pub tiles: Vec<Tile>,
    pub units: Vec<Unit>,
    pub bases: Vec<Base>,
    pub factions: Vec<Faction>,
    pub log: Vec<String>,
    pub game_over: Option<GameOver>,
}

impl GameState {
    pub fn new_game(width: usize, height: usize, seed: u32) -> Self {
        let mut state = Self {
            width,
            height,
            seed,
            turn: 1,
            tiles: Vec::new(),
            units: Vec::new(),
            bases: Vec::new(),
            factions: vec![
                Faction {
                    id: PLAYER_ID,
                    name: "Gaia's Stepdaughters".to_string(),
                    energy: 10,
                    research: 0,
                    techs_discovered: 0,
                    is_ai: false,
                },
                Faction {
                    id: AI_ID,
                    name: "Spartan Federation".to_string(),
                    energy: 10,
                    research: 0,
                    techs_discovered: 0,
                    is_ai: true,
                },
            ],
            log: Vec::new(),
            game_over: None,
        };

        state.generate_map();

        let player_start = (3usize, 3usize);
        let ai_start = (width.saturating_sub(5), height.saturating_sub(5));

        state.force_land_patch(player_start.0, player_start.1);
        state.force_land_patch(ai_start.0, ai_start.1);

        state.add_unit(
            PLAYER_ID,
            UnitKind::ColonyPod,
            player_start.0,
            player_start.1,
        );
        state.add_unit(
            PLAYER_ID,
            UnitKind::ScoutPatrol,
            player_start.0 + 1,
            player_start.1,
        );

        state.add_unit(AI_ID, UnitKind::ColonyPod, ai_start.0, ai_start.1);
        state.add_unit(
            AI_ID,
            UnitKind::ScoutPatrol,
            ai_start.0.saturating_sub(1),
            ai_start.1,
        );

        state.push_log("MISSION YEAR 2101: Planetfall confirmed.".to_string());
        state.push_log(
            "Objective: found bases, scout Planet, and defeat the rival faction.".to_string(),
        );

        state
    }

    pub fn push_log(&mut self, message: String) {
        self.log.push(message);
        if self.log.len() > 80 {
            self.log.remove(0);
        }
    }

    pub fn tile(&self, x: usize, y: usize) -> Option<&Tile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.get(self.idx(x, y))
    }

    pub fn unit(&self, id: usize) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id && u.alive)
    }

    pub fn base(&self, id: usize) -> Option<&Base> {
        self.bases.iter().find(|b| b.id == id)
    }

    pub fn live_units_for(&self, owner: usize) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|u| u.alive && u.owner == owner)
            .collect()
    }

    pub fn bases_for(&self, owner: usize) -> Vec<&Base> {
        self.bases.iter().filter(|b| b.owner == owner).collect()
    }

    pub fn base_yields(&self, x: usize, y: usize) -> Yields {
        let mut total = Yields::default();

        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                let nx = x as isize + dx;
                let ny = y as isize + dy;

                if nx < 0 || ny < 0 {
                    continue;
                }

                if let Some(tile) = self.tile(nx as usize, ny as usize) {
                    total = total.add(tile.terrain.yields());
                }
            }
        }

        total
    }

    pub fn found_base(&mut self, unit_id: usize) -> Result<(), String> {
        if self.game_over.is_some() {
            return Err("Game is already over.".to_string());
        }

        let unit = self
            .units
            .iter()
            .find(|u| u.id == unit_id && u.alive)
            .cloned()
            .ok_or_else(|| "Selected unit no longer exists.".to_string())?;

        if !unit.kind.can_found_base() {
            return Err("Only colony pods can found bases.".to_string());
        }

        let idx = self.idx(unit.x, unit.y);

        if !self.tiles[idx].terrain.is_land() {
            return Err("Cannot found a base in the ocean.".to_string());
        }

        if self.tiles[idx].base.is_some() {
            return Err("There is already a base here.".to_string());
        }

        let base_id = self.bases.len();
        let name = self.next_base_name(unit.owner);

        self.bases.push(Base {
            id: base_id,
            owner: unit.owner,
            name: name.clone(),
            x: unit.x,
            y: unit.y,
            population: 1,
            nutrients_stock: 0,
            minerals_stock: 0,
            production: ProductionItem::ScoutPatrol,
        });

        self.tiles[idx].base = Some(base_id);
        self.destroy_unit(unit_id);

        let faction_name = self.factions[unit.owner].name.clone();
        self.push_log(format!("{faction_name} founded {name}."));
        self.check_game_over();

        Ok(())
    }

    pub fn move_unit_to(
        &mut self,
        unit_id: usize,
        target_x: usize,
        target_y: usize,
    ) -> Result<(), String> {
        if self.game_over.is_some() {
            return Err("Game is already over.".to_string());
        }

        if target_x >= self.width || target_y >= self.height {
            return Err("Target is outside the map.".to_string());
        }

        let unit = self
            .units
            .iter()
            .find(|u| u.id == unit_id && u.alive)
            .cloned()
            .ok_or_else(|| "Selected unit no longer exists.".to_string())?;

        if unit.moves_left <= 0 {
            return Err("That unit has no moves left this turn.".to_string());
        }

        if !Self::is_adjacent(unit.x, unit.y, target_x, target_y) {
            return Err("Units can only move one tile at a time.".to_string());
        }

        let target_idx = self.idx(target_x, target_y);
        let target_tile = self.tiles[target_idx].clone();

        if !target_tile.terrain.is_land() {
            return Err("Land units cannot enter ocean yet.".to_string());
        }

        if let Some(defender_id) = target_tile.unit {
            let defender = self
                .units
                .iter()
                .find(|u| u.id == defender_id && u.alive)
                .cloned()
                .ok_or_else(|| "Target unit state is corrupt.".to_string())?;

            if defender.owner == unit.owner {
                return Err("Another friendly unit is already on that tile.".to_string());
            }

            self.resolve_combat(unit_id, defender_id, target_x, target_y);
            self.check_game_over();
            return Ok(());
        }

        self.move_unit_without_combat(unit_id, target_x, target_y)?;

        if let Some(base_id) = self.tiles[target_idx].base {
            let owner = self.bases[base_id].owner;
            if owner != unit.owner {
                self.bases[base_id].owner = unit.owner;
                let faction_name = self.factions[unit.owner].name.clone();
                let base_name = self.bases[base_id].name.clone();
                self.push_log(format!("{faction_name} captured {base_name}!"));
            }
        }

        self.check_game_over();
        Ok(())
    }

    pub fn end_turn(&mut self) {
        if self.game_over.is_some() {
            return;
        }

        self.turn += 1;
        self.push_log(format!("--- Mission Year {} ---", 2100 + self.turn));

        self.collect_from_bases();
        self.reset_moves(AI_ID);
        self.run_ai_turn();
        self.collect_from_bases();
        self.reset_moves(PLAYER_ID);

        self.check_game_over();
    }

    fn generate_map(&mut self) {
        self.tiles.clear();

        for y in 0..self.height {
            for x in 0..self.width {
                let altitude = self.noise(x as i32, y as i32, 17) % 100;
                let moisture = self.noise(x as i32, y as i32, 93) % 100;
                let rock = self.noise(x as i32, y as i32, 211) % 100;
                let fungus = self.noise(x as i32, y as i32, 501) % 100;

                let terrain = if altitude < 18 {
                    Terrain::Ocean
                } else if fungus > 86 {
                    Terrain::Fungus
                } else if rock > 76 {
                    Terrain::Rocky
                } else if moisture > 52 {
                    Terrain::Flat
                } else {
                    Terrain::Rolling
                };

                self.tiles.push(Tile {
                    x,
                    y,
                    terrain,
                    elevation: altitude as i32,
                    moisture: moisture as i32,
                    unit: None,
                    base: None,
                });
            }
        }
    }

    fn force_land_patch(&mut self, cx: usize, cy: usize) {
        for dy in -3isize..=3 {
            for dx in -3isize..=3 {
                let nx = cx as isize + dx;
                let ny = cy as isize + dy;

                if nx < 0 || ny < 0 {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;

                if nx >= self.width || ny >= self.height {
                    continue;
                }

                let idx = self.idx(nx, ny);
                self.tiles[idx].terrain = if (dx.abs() + dy.abs()) % 3 == 0 {
                    Terrain::Rolling
                } else {
                    Terrain::Flat
                };
            }
        }
    }

    fn add_unit(&mut self, owner: usize, kind: UnitKind, x: usize, y: usize) -> usize {
        let id = self.units.len();
        let idx = self.idx(x, y);

        self.units.push(Unit {
            id,
            owner,
            kind,
            x,
            y,
            moves_left: kind.max_moves(),
            hp: 10,
            alive: true,
        });

        self.tiles[idx].unit = Some(id);
        id
    }

    fn spawn_unit_near(&mut self, owner: usize, kind: UnitKind, x: usize, y: usize) -> bool {
        let candidates = [
            (x, y),
            (x.saturating_add(1), y),
            (x.saturating_sub(1), y),
            (x, y.saturating_add(1)),
            (x, y.saturating_sub(1)),
            (x.saturating_add(1), y.saturating_add(1)),
            (x.saturating_sub(1), y.saturating_sub(1)),
        ];

        for (nx, ny) in candidates {
            if nx >= self.width || ny >= self.height {
                continue;
            }

            let idx = self.idx(nx, ny);
            let tile = &self.tiles[idx];

            if tile.terrain.is_land() && tile.unit.is_none() {
                self.add_unit(owner, kind, nx, ny);
                return true;
            }
        }

        false
    }

    fn collect_from_bases(&mut self) {
        let base_ids: Vec<usize> = self.bases.iter().map(|b| b.id).collect();

        for base_id in base_ids {
            let x = self.bases[base_id].x;
            let y = self.bases[base_id].y;
            let owner = self.bases[base_id].owner;
            let yields = self.base_yields(x, y);

            self.bases[base_id].nutrients_stock += yields.nutrients;
            self.bases[base_id].minerals_stock += yields.minerals;
            self.factions[owner].energy += yields.energy;
            self.factions[owner].research += yields.energy;

            if self.bases[base_id].nutrients_stock >= 20 {
                self.bases[base_id].nutrients_stock -= 20;
                self.bases[base_id].population += 1;
                let name = self.bases[base_id].name.clone();
                self.push_log(format!(
                    "{name} grew to population {}.",
                    self.bases[base_id].population
                ));
            }

            let item = self.bases[base_id].production;
            if self.bases[base_id].minerals_stock >= item.cost() {
                self.bases[base_id].minerals_stock -= item.cost();

                if self.spawn_unit_near(owner, item.unit_kind(), x, y) {
                    let name = self.bases[base_id].name.clone();
                    self.push_log(format!("{name} completed production: {}.", item.name()));
                }
            }

            if self.factions[owner].research >= 50 {
                self.factions[owner].research -= 50;
                self.factions[owner].techs_discovered += 1;
                let faction = self.factions[owner].name.clone();
                self.push_log(format!("{faction} discovered a new technology."));
            }
        }
    }

    fn run_ai_turn(&mut self) {
        let ai_colonies: Vec<usize> = self
            .units
            .iter()
            .filter(|u| u.alive && u.owner == AI_ID && u.kind == UnitKind::ColonyPod)
            .map(|u| u.id)
            .collect();

        if self.bases_for(AI_ID).is_empty() {
            for colony_id in ai_colonies {
                let _ = self.found_base(colony_id);
                break;
            }
        }

        let ai_units: Vec<usize> = self
            .units
            .iter()
            .filter(|u| u.alive && u.owner == AI_ID)
            .map(|u| u.id)
            .collect();

        for unit_id in ai_units {
            let Some(unit) = self.unit(unit_id).cloned() else {
                continue;
            };

            if unit.moves_left <= 0 {
                continue;
            }

            if let Some((tx, ty)) = self.nearest_player_target(unit.x, unit.y) {
                let step_x = step_toward(unit.x, tx);
                let step_y = step_toward(unit.y, ty);

                let nx = (unit.x as isize + step_x).clamp(0, self.width.saturating_sub(1) as isize)
                    as usize;
                let ny = (unit.y as isize + step_y).clamp(0, self.height.saturating_sub(1) as isize)
                    as usize;

                if nx != unit.x || ny != unit.y {
                    let _ = self.move_unit_to(unit_id, nx, ny);
                }
            }
        }
    }

    fn nearest_player_target(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize, usize)> = None;

        for unit in self
            .units
            .iter()
            .filter(|u| u.alive && u.owner == PLAYER_ID)
        {
            let distance = manhattan(x, y, unit.x, unit.y);
            if best.map(|b| distance < b.2).unwrap_or(true) {
                best = Some((unit.x, unit.y, distance));
            }
        }

        for base in self.bases.iter().filter(|b| b.owner == PLAYER_ID) {
            let distance = manhattan(x, y, base.x, base.y);
            if best.map(|b| distance < b.2).unwrap_or(true) {
                best = Some((base.x, base.y, distance));
            }
        }

        best.map(|b| (b.0, b.1))
    }

    fn resolve_combat(
        &mut self,
        attacker_id: usize,
        defender_id: usize,
        target_x: usize,
        target_y: usize,
    ) {
        let Some(attacker) = self.unit(attacker_id).cloned() else {
            return;
        };
        let Some(defender) = self.unit(defender_id).cloned() else {
            return;
        };

        let roll = (self.noise(attacker.x as i32, defender.y as i32, self.turn as u32) % 6) as i32;
        let attack_score = attacker.kind.attack() + roll + 1;
        let defense_score = defender.kind.defense() + defender.hp / 4;

        let attacker_name = self.factions[attacker.owner].name.clone();
        let defender_name = self.factions[defender.owner].name.clone();

        if attack_score >= defense_score {
            self.destroy_unit(defender_id);
            let _ = self.move_unit_without_combat(attacker_id, target_x, target_y);
            self.set_unit_moves(attacker_id, 0);
            self.push_log(format!(
                "{attacker_name} {} destroyed {defender_name} {}.",
                attacker.kind.name(),
                defender.kind.name()
            ));
        } else {
            self.destroy_unit(attacker_id);
            self.push_log(format!(
                "{attacker_name} {} was destroyed attacking {defender_name} {}.",
                attacker.kind.name(),
                defender.kind.name()
            ));
        }
    }

    fn move_unit_without_combat(
        &mut self,
        unit_id: usize,
        target_x: usize,
        target_y: usize,
    ) -> Result<(), String> {
        let unit = self
            .units
            .iter()
            .find(|u| u.id == unit_id && u.alive)
            .cloned()
            .ok_or_else(|| "Selected unit no longer exists.".to_string())?;

        let from_idx = self.idx(unit.x, unit.y);
        let target_idx = self.idx(target_x, target_y);

        self.tiles[from_idx].unit = None;
        self.tiles[target_idx].unit = Some(unit_id);

        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
            unit.x = target_x;
            unit.y = target_y;
            unit.moves_left -= 1;
        }

        Ok(())
    }

    fn destroy_unit(&mut self, unit_id: usize) {
        let unit = self.units.iter().find(|u| u.id == unit_id).cloned();

        if let Some(unit) = unit {
            if unit.x < self.width && unit.y < self.height {
                let idx = self.idx(unit.x, unit.y);
                if self.tiles[idx].unit == Some(unit_id) {
                    self.tiles[idx].unit = None;
                }
            }
        }

        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id) {
            unit.alive = false;
            unit.moves_left = 0;
            unit.hp = 0;
        }
    }

    fn set_unit_moves(&mut self, unit_id: usize, moves: i32) {
        if let Some(unit) = self.units.iter_mut().find(|u| u.id == unit_id && u.alive) {
            unit.moves_left = moves;
        }
    }

    fn reset_moves(&mut self, owner: usize) {
        for unit in self
            .units
            .iter_mut()
            .filter(|u| u.alive && u.owner == owner)
        {
            unit.moves_left = unit.kind.max_moves();
        }
    }

    fn check_game_over(&mut self) {
        let player_alive = self.faction_alive(PLAYER_ID);
        let ai_alive = self.faction_alive(AI_ID);

        if !player_alive {
            self.game_over = Some(GameOver::PlayerLost);
            self.push_log("DEFEAT: Your faction has been wiped out.".to_string());
        } else if !ai_alive {
            self.game_over = Some(GameOver::PlayerWon);
            self.push_log("VICTORY: Rival faction eliminated.".to_string());
        }
    }

    fn faction_alive(&self, owner: usize) -> bool {
        self.units.iter().any(|u| u.alive && u.owner == owner)
            || self.bases.iter().any(|b| b.owner == owner)
    }

    fn next_base_name(&self, owner: usize) -> String {
        let count = self.bases.iter().filter(|b| b.owner == owner).count() + 1;

        if owner == PLAYER_ID {
            match count {
                1 => "Landing Point".to_string(),
                2 => "Greenhouse Gate".to_string(),
                3 => "Planetfall Nexus".to_string(),
                _ => format!("Gaian Base {count}"),
            }
        } else {
            match count {
                1 => "Sparta Command".to_string(),
                2 => "Fort Survival".to_string(),
                3 => "Ironholm".to_string(),
                _ => format!("Spartan Base {count}"),
            }
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn is_adjacent(ax: usize, ay: usize, bx: usize, by: usize) -> bool {
        let dx = ax.abs_diff(bx);
        let dy = ay.abs_diff(by);
        dx <= 1 && dy <= 1 && (dx + dy) > 0
    }

    fn noise(&self, x: i32, y: i32, salt: u32) -> u32 {
        let mut n = self.seed
            ^ salt
            ^ ((x as u32).wrapping_mul(374_761_393))
            ^ ((y as u32).wrapping_mul(668_265_263));

        n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
        n ^ (n >> 16)
    }
}

fn step_toward(current: usize, target: usize) -> isize {
    if target > current {
        1
    } else if target < current {
        -1
    } else {
        0
    }
}

fn manhattan(ax: usize, ay: usize, bx: usize, by: usize) -> usize {
    ax.abs_diff(bx) + ay.abs_diff(by)
}
