pub struct Textured(pub ggez::graphics::Rect);

pub struct BlockType(pub u32);
pub struct Position(pub u16, pub u16);
pub struct Table(pub Option<mlua::RegistryKey>);

pub struct PowerProducer(pub f32);
pub struct PowerConsumer(pub f32);
pub struct NetworkId(pub u32);

pub type ECS = hecs::World;
