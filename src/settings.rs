pub struct Settings {
    pub sc_width: f32,
    pub sc_height: f32,
    pub fullscreen_type: ggez::conf::FullscreenType,
    pub mouse_wheel_sensitivity: f32,
}

impl Settings {
    pub fn new(sc_width: f32, sc_height: f32) -> Self {
        Settings {
            sc_width,
            sc_height,
            fullscreen_type: ggez::conf::FullscreenType::Windowed,
            mouse_wheel_sensitivity: 0.5,
        }
    }
}

pub mod lua {
    pub mod param {
        // field names for scripts
        pub const STRING_ID: &str = "str_id";
        pub const BLOCK_INDEX_IN_REGISTRY: &str = "raw_id";
        pub const ENTITY_ID: &str = "entity_id";
        pub const NETWORK_ID: &str = "net_id";
        pub const POSITION: &str = "pos";
    }

    pub mod functions {
        pub const INIT: &str = "init";
        pub const UPDATE: &str = "update";
        pub const MOUSE_BUTTON_DOWN: &str = "on_mouse_button_down";
        pub const MOUSE_BUTTON_UP: &str = "on_mouse_button_up";
    }
}

pub mod json {
    pub mod fields {
        // parameter names for .json block definitions
        pub const ENERGY_POWER: &str = "power";
        pub const ENERGY_DEMAND: &str = "demand";
        pub const ENERGY_MASK: &str = "mask";
    }

    pub mod mask {
        pub const PRODUCER: u8 = 1;
        pub const CONSUMER: u8 = 2;
        pub const STORAGE: u8 = 3;
        pub const NODE: u8 = 4;
    }
}

pub mod res {
    pub const MISSING_TEX: &str = "./resources/assets/textures/missing.png";
    pub const TEXTURE_SIZE: f32 = 32.0;
}
