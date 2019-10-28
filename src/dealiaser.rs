use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

#[derive(Debug, Clone)]
pub struct Dealiaser {
    classes: Arc<RwLock<HashMap<String, String>>>,
}

impl Dealiaser {
    pub fn new() -> Dealiaser {
        let classes: HashMap<String, String> = HashMap::new();
        let c = Arc::new(RwLock::new(classes));
        Dealiaser { classes: c }
    }

    pub fn add(&mut self, class: &str, alias: &str) {
        let mut c = self.classes.write().unwrap();
        c.insert(alias.to_owned(), class.to_owned());
    }

    pub fn dealias(&mut self, alias: &str) -> Option<String> {
        let c = self.classes.read().unwrap();
        match c.get(alias) {
            Some(c) => Some(c.to_owned()),
            None => None,
        }
    }

    pub fn add_from_yml(&mut self, file_path: &str) {
        let mut file = File::open(file_path).unwrap(); // check err
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let yaml = match YamlLoader::load_from_str(&contents) {
            Ok(y) => y,
            Err(e) => {
                eprintln!("/!\\Yaml error in `{}`: {}", file_path, e);
                return;
            }
        };
        let services = match &yaml[0]["services"] {
            Yaml::BadValue => return,
            s => s.as_hash().unwrap(),
        };
        let mut alias_map = self.classes.write().unwrap();
        for (s_name, s_opts) in services {
            let s_name = s_name.as_str().unwrap();
            let s_alias = {
                if let Some(s_alias) = s_opts["alias"].as_str() {
                    s_alias
                } else if let Some(s_alias) = s_opts["class"].as_str() {
                    s_alias
                } else {
                    return;
                }
            };
            let (namespaced, pointed) = match s_name.contains("\\") {
                true => (s_name, s_alias),
                false => (s_alias, s_name),
            };
            alias_map.insert(pointed.to_owned(), namespaced.to_owned());
        }
        drop(alias_map);
    }
}
