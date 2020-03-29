use crate::grid;
use amethyst::{
    assets::Handle,
    core::timing::Time,
    core::{SystemDesc, Transform},
    derive::SystemDesc,
    ecs::{
        prelude::{Component, DenseVecStorage},
        Join, Read, System, SystemData, World, Write, WriteStorage,
    },
    prelude::*,
    renderer::{SpriteRender, SpriteSheet},
};
use anyhow::{bail, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Default)]
pub struct GridState {
    pub last_tick: Option<Duration>,
    pub tiles_current: TileTypeGrid,
    pub tiles_old: TileTypeGrid,
    pub sprites: Option<Handle<SpriteSheet>>,
    pub player_pos: GridPos,
}

#[derive(Default)]
pub struct GridPos {
    pub x: usize,
    pub y: usize,
}

impl GridPos {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Component for GridPos {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemDesc)]
pub struct GridObjectSystem;

impl<'s> System<'s> for GridObjectSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, GridPos>,
        Read<'s, Time>,
        Write<'s, GridState>,
    );

    fn run(
        &mut self,
        (mut transforms, mut grid_objects, time, mut grid_map_state): Self::SystemData,
    ) {
        if grid_map_state
            .last_tick
            .map(|last| time.absolute_time() - last < Duration::from_millis(5000))
            .unwrap_or(false)
        {
            return;
        } else {
            grid_map_state.last_tick = Some(time.absolute_time());
        }

        for (grid_object, transform) in (&mut grid_objects, &mut transforms).join() {
            let x = &mut grid_object.x;
            let y = &mut grid_object.y;
            let GridState {
                ref tiles_current,
                ref mut tiles_old,
                ..
            } = *grid_map_state;
            // let tiles = &grid_map_state.tiles_current;
            // let new_tiles = &mut grid_map_state.tiles_old;

            /*
            let type_ = tiles_current.get(*x, *y);

            if type_.is_falling() {
                let type_below = tiles_current.get(*x, *y - 1);
                if type_below.is_empty() {
                    *tiles_old.get_mut(*x, *y) = type_below;
                    *tiles_old.get_mut(*x, *y - 1) = type_;
                    *y = *y - 1;
                }
            }
            */

            transform.set_translation_y(*y as f32 * 32.);
            transform.set_translation_x(*x as f32 * 32.);
        }
        /*
        let GridState {
            ref mut tiles_current,
            ref mut tiles_old,
            ..
        } = *grid_map_state;
        std::mem::swap(tiles_current, tiles_old);
        */
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum TileType {
    Empty,
    Player,
    Dirt,
    Rock,
    Wall,
}

impl TileType {
    fn to_sprite_number(self) -> Option<usize> {
        use TileType::*;
        Some(match self {
            Empty => return None,
            Player => 0,
            Dirt => 2,
            Rock => 3,
            Wall => 4,
        })
    }

    fn is_falling(self) -> bool {
        use TileType::*;
        match self {
            Rock => true,
            _ => false,
        }
    }
    fn is_empty(self) -> bool {
        use TileType::*;
        match self {
            Empty => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone)]
pub struct TileTypeGrid {
    height: usize,
    width: usize,
    tiles: Vec<TileType>,
}

impl TileTypeGrid {
    fn get(&self, x: usize, y: usize) -> TileType {
        *self.get_ref(x, y)
    }

    fn get_ref(&self, x: usize, y: usize) -> &TileType {
        &self.tiles[x + (self.height - y - 1) * self.width]
    }
    fn get_mut(&mut self, x: usize, y: usize) -> &mut TileType {
        &mut self.tiles[x + (self.height - y - 1) * self.width]
    }
}

struct LoadMapData {
    grid: TileTypeGrid,
    start: GridPos,
}

fn load_map(path: PathBuf) -> Result<LoadMapData> {
    let mut width = None;
    let mut height = 0;
    let mut start = None;
    let mut tiles = vec![];

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (y, line) in reader.lines().enumerate() {
        let line = line?;
        if let Some(width) = width {
            if width != line.len() {
                bail!("Lines not equal len");
            }
        } else {
            width = Some(line.len());
        }
        height = y + 1;

        for (x, ch) in line.chars().enumerate() {
            tiles.push(match ch {
                's' => {
                    start = Some(GridPos::new(x, y));
                    TileType::Player
                }
                '#' => TileType::Wall,
                '.' => TileType::Dirt,
                'o' => TileType::Rock,
                _ => TileType::Empty,
            });
        }
    }

    Ok(LoadMapData {
        grid: TileTypeGrid {
            width: width.unwrap(),
            height,
            tiles,
        },
        start: start.unwrap(),
    })
}

pub fn init(world: &mut World, sprites: Handle<SpriteSheet>) {
    let LoadMapData { grid, start } = load_map("./resources/map/01.txt".into()).unwrap();

    let state = GridState {
        last_tick: Default::default(),
        tiles_current: grid.clone(),
        tiles_old: grid.clone(),
        sprites: Some(sprites.clone()),
        player_pos: start,
    };

    for y in 0..grid.height {
        for x in 0..grid.width {
            let t = grid.get(x, y);
            if let Some(sprite_number) = t.to_sprite_number() {
                let sprite_render = SpriteRender {
                    sprite_sheet: sprites.clone(),
                    sprite_number,
                };

                world
                    .create_entity()
                    .with(sprite_render)
                    .with(grid::GridPos { y, x })
                    .with(Transform::default())
                    .build();
            }
        }
    }

    world.register::<grid::GridPos>();
    world.insert(state);
}
