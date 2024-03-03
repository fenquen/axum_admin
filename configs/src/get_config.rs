use std::{fs::File, io::Read};

use once_cell::sync::Lazy;

use super::cfgs::Configs;

const CFG_FILE: &str = "config/config.toml";

pub static CFG: Lazy<Configs> = Lazy::new(Configs::init);

impl Configs {
    pub fn init() -> Self {
        let mut file = match File::open(CFG_FILE) {
            Ok(f) => f,
            Err(e) => panic!("不存在配置文件：{}，错误信息：{}", CFG_FILE, e),
        };

        let mut cfg_contents = String::new();

        match file.read_to_string(&mut cfg_contents) {
            Ok(s) => s,
            Err(e) => panic!("读取配置文件失败，错误信息：{}", e),
        };

        toml::from_str(&cfg_contents).expect("解析配置文件错误")
    }
}
