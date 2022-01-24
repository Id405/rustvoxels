use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum ConfigValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
}

impl ConfigValue {
    pub fn as_f32(&self) -> f32 {
        match self {
            ConfigValue::I32(i) => *i as f32,
            ConfigValue::I64(i) => *i as f32,
            ConfigValue::F32(f) => *f as f32,
            ConfigValue::F64(f) => *f as f32,
            ConfigValue::String(s) => s.parse::<f32>().unwrap(),
            _ => panic!(),
        }
    }
    pub fn as_i32(&self) -> i32 {
        match self {
            ConfigValue::Bool(b) => *b as i32,
            ConfigValue::I32(i) => *i,
            ConfigValue::I64(i) => *i as i32,
            ConfigValue::F32(f) => *f as i32,
            ConfigValue::F64(f) => *f as i32,
            ConfigValue::String(s) => s.parse::<i32>().unwrap(),
        }
    }
    pub fn as_bool(&self) -> bool {
        match self {
            ConfigValue::Bool(b) => *b,
            ConfigValue::I32(i) => *i != 0,
            ConfigValue::I64(i) => *i != 0,
            ConfigValue::F32(f) => *f != 0.0,
            ConfigValue::F64(f) => *f != 0.0,
            ConfigValue::String(s) => s.parse::<bool>().unwrap(),
        }
    }
}

pub struct Config {
    config_vars: HashMap<String, ConfigValue>,
}

impl Config {
    pub fn new() -> Self {
        let mut config = Config {
            config_vars: HashMap::new(),
        };

        config.initialize();
        config
    }

    pub fn initialize(&mut self) {
        self.set_var("renderer_raytracer_samples", ConfigValue::I32(1));
        self.set_var("renderer_raytracer_do_lighting", ConfigValue::Bool(false));
        self.set_var("renderer_raytracer_max_steps", ConfigValue::I32(200));
        self.set_var(
            "renderer_denoiser_enable_filtering",
            ConfigValue::Bool(true),
        );
        self.set_var(
            "renderer_denoiser_reprojection_percent",
            ConfigValue::F32(0.90),
        );
        self.set_var(
            "renderer_denoiser_edge_avoiding_blur_strength",
            ConfigValue::F32(1.5),
        );
        self.set_var("renderer_fov", ConfigValue::F32(90.0));
        self.set_var("game_input_mouse_sensitivity", ConfigValue::F32(0.0005));
        self.set_var("game_input_movement_speed", ConfigValue::F32(50.0));
        self.set_var(
            "game_world_file_path",
            ConfigValue::String("garfield.vox".to_string()),
        );
        self.set_var("game_enable_editor", ConfigValue::Bool(false));
        // Editor mode opens up additional controls to easily control do_lighting
        // movement speed, render entities, and edit the world + voxels.
    }

    pub fn set_var(&mut self, name: &str, value: ConfigValue) {
        // TODO, make generic
        self.config_vars.insert(name.to_string(), value);
    }

    pub fn get_var(&self, name: &str) -> Option<ConfigValue> {
        self.config_vars.get(name).map(|x| x.clone())
    }
}
