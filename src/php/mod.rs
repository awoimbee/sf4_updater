use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug)]
struct Class {
    children: Vec<String>, // pointers would be better
    parent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Php {
    classes: Arc<RwLock<HashMap<String, Class>>>,
}

impl Php {
    pub fn new() -> Php {
        let h_map: HashMap<String, Class> = HashMap::new();
        let c = Arc::new(RwLock::new(h_map));
        Php { classes: c }
    }

    pub fn add_from_php(&mut self, file_path: &str) {
        lazy_static! {
            static ref RE_CLASS: Regex =
                Regex::new(r"class ([^ \n]*)( extends ([^ \n]*))?").unwrap();
        }
        let mut file = File::open(file_path).unwrap(); // check err
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let caps = match RE_CLASS.captures(&contents) {
            Some(c) => c,
            None => {
                println!("Not recognised as a class {}", file_path);
                return;
            }
        };
        let class = caps.get(1).map_or("", |m| m.as_str()); // class
        let parent = caps.get(2).map_or("", |m| m.as_str()); // parent

        let mut map_writer = self.classes.write().unwrap();
        // map_writer.

        println!("Class: {:50}\tParent: {}", class, parent);
    }
}
