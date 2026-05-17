use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Ocean,
    Land,
    Hills,
    Mountain,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub value: f32, // The Perlin noise value
    pub tile_type: TileType,
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn generate(width: usize, height: usize, seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        let mut tiles = vec![
            vec![
                Tile {
                    value: 0.0,
                    tile_type: TileType::Ocean
                };
                width
            ];
            height
        ];
        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;
                let value = perlin.get([nx * 8.0, ny * 8.0]) as f32;

                let tile_type = if value < -0.2 {
                    TileType::Ocean
                } else if value < 0.1 {
                    TileType::Land
                } else if value < 0.4 {
                    TileType::Hills
                } else {
                    TileType::Mountain
                };

                tiles[y][x] = Tile { value, tile_type };
            }
        }
        Map {
            width,
            height,
            tiles,
        }
    }

    pub fn print_ascii(&self) {
        for row in &self.tiles {
            for tile in row {
                print!(
                    "{}",
                    match tile.tile_type {
                        TileType::Ocean => '~',
                        TileType::Land => '.',
                        TileType::Hills => '#',
                        TileType::Mountain => '^',
                    }
                );
            }
            println!();
        }
    }
}
