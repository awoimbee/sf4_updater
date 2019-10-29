use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug)]
struct Class {
    children: Vec<String>,
    parent: Option<String>,
    uses: HashMap<String, String>, // Name \ Class
    path: String,
}

#[derive(Debug, Clone)]
pub struct Php {
    classes: Arc<RwLock<HashMap<String, Class>>>,
}

impl Class {
    pub fn new() -> Class {
        let children = Vec::new();
        let parent = None;
        let uses = HashMap::new();
        let path = String::new();
        Class {
            children,
            parent,
            uses,
            path,
        }
    }
}

impl Php {
    pub fn new() -> Php {
        let h_map: HashMap<String, Class> = HashMap::new();
        let c = Arc::new(RwLock::new(h_map));
        Php { classes: c }
    }
}

mod extract_php;
mod remove_get;
