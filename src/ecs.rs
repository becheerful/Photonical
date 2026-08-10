#[derive(Debug, Clone)]
pub struct Textured(pub ggez::graphics::Rect);

pub struct BlockType(pub u32);

#[derive(Debug, Clone)]
pub struct Position(pub u16, pub u16);
impl Position {
    pub fn to_vec2(&self) -> ggez::glam::Vec2 {
        ggez::glam::Vec2::new(self.0 as f32, self.1 as f32)
    }
}

pub struct Table(pub Option<mlua::RegistryKey>);

pub struct PowerProducer(pub f32);
pub struct PowerConsumer(pub f32);
pub struct NetworkId(pub u32);

pub type ECS = hecs::World;
