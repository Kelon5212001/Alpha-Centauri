use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub enum Terrain {
    Ocean,
    Land,
    Fungus,
    Mountain,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub terrain: Terrain,
}

pub fn generate_map(width: usize, height: usize) -> Vec<Tile> {
    let mut rng = rand::thread_rng();
    let mut map = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let terrain = match rng.gen_range(0..100) {
                0..=49 => Terrain::Ocean,
                50..=85 => Terrain::Land,
                86..=94 => Terrain::Fungus,
                _ => Terrain::Mountain,
            };
            map.push(Tile { x, y, terrain });
        }
    }
    map
}
