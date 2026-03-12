use crate::cli::ResizeFilter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub mode: String,
    pub latency: bool,
    pub filter: ResizeFilter,
    pub full: bool,
    pub index: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: "ansi".into(),
            latency: false,
            filter: ResizeFilter::Lanczos3,
            full: false,
            index: "index.json".into(),
        }
    }
}
