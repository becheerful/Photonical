use std::collections::HashMap;

use ggez::glam::UVec2;
use hecs::Entity;

use crate::CHUNK_SIZE;

pub struct BlockType(pub u32);
pub struct Position(pub UVec2);
pub struct Scripted;

#[derive(Clone)]
pub struct Chunk {
    blocks: [[Option<Entity>; CHUNK_SIZE]; CHUNK_SIZE],
}

pub struct World {
    pub ecs: hecs::World,
    pub width: u16,
    pub height: u16,
    pub tile_size: f32,
    pub chunks: HashMap<UVec2, Chunk>,
}

impl World {
    pub fn new(width: u16, height: u16, tile_size: f32) -> Self {
        let mut ecs = hecs::World::new();
        let mut chunks = HashMap::new();

        let u32width = width as u32;
        let u32height = height as u32;

        for x in 0..u32width {
            for y in 0..u32height {
                // `as` is justified, as `CHUNK_SIZE` is guaranteed to be less than 65535
                let chunk_x = x / (CHUNK_SIZE as u32);
                let chunk_y = y / (CHUNK_SIZE as u32);
                let local_x = x as usize % CHUNK_SIZE;
                let local_y = y as usize % CHUNK_SIZE;

                let entity = ecs.spawn((
                    /*
                        Nobody doesn't really know which block is indexed by 0.
                        It can be just a stone or a nuclear bomb.
                        I hope I spell the identifiers correctly...
                                                    ...or something bad will happen. 💀

                        TODO: rewrite for the safety of the world
                    */
                    BlockType(crate::defs::registry().get_block_index("photonical:stone").unwrap_or(0)),
                    Position(UVec2::new(x, y)),
                ));

                let chunk_key = UVec2::new(chunk_x, chunk_y);
                let chunk = chunks.entry(chunk_key).or_insert_with(|| Chunk {
                    blocks: [[None; CHUNK_SIZE]; CHUNK_SIZE]
                });

                chunk.blocks[local_x][local_y] = Some(entity);
            }
        }

        World {
            ecs,
            width,
            height,
            tile_size,
            chunks,
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<Entity> {
        // `as` is justified, as `CHUNK_SIZE` is guaranteed to be less than 65535
        let chunk_x = x.div_euclid(CHUNK_SIZE as u16);
        let chunk_y = y.div_euclid(CHUNK_SIZE as u16);

        let local_x = (x as usize).rem_euclid(CHUNK_SIZE);
        let local_y = (y as usize).rem_euclid(CHUNK_SIZE);

        self.chunks
            .get(&UVec2::new(chunk_x as u32, chunk_y as u32))
            .and_then(|chunk| chunk.blocks[local_x][local_y])
    }
}
