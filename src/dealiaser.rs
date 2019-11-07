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

    pub fn add(&mut self, psr: &str, alias: &str) {
        let mut c = self.classes.write().unwrap();
        c.insert(alias.to_owned(), psr.to_owned());
    }

    pub fn dealias(&self, alias: &str) -> Option<String> {
        let c = self.classes.read().unwrap();
        match c.get(alias) {
            Some(dealiased) => Some(dealiased.to_owned()),
            None => None,
        }
    }

    pub fn add_from_yml(&mut self, file_path: &str) {
        // println!("Add from yml conf from: {}", file_path);
        let mut file = File::open(file_path).unwrap();
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
        for (s_name, s_opts) in services {
            let s_name = s_name.as_str().unwrap();
            let s_alias = match s_opts["alias"].as_str() {
                Some(s_alias) => s_alias,
                None => match s_opts["class"].as_str() {
                    Some(s_psr) => s_psr,
                    None => continue,
                },
            };
            let (namespaced, pointed) = match s_name.contains("\\") {
                true => (s_name, s_alias),
                false => (s_alias, s_name),
            };
            let mut alias_map = self.classes.write().unwrap();
            alias_map.insert(pointed.to_owned(), namespaced.to_owned());
        }
    }

    /// Needed for services w/ multiple aliases as they may use diff. args.
    pub fn checkup(&mut self) {
        let mut rev_map: HashMap<&str, &str> = HashMap::new();
        let mut map_w = self.classes.write().unwrap();

        let mut to_remove: Vec<String> = Vec::new();
        for (alias, namespace) in map_w.iter() {
            if rev_map.contains_key(namespace.as_str()) {
                let old_alias = rev_map.get(namespace.as_str()).unwrap();
                to_remove.push(old_alias.to_string());
                to_remove.push(alias.to_owned());
                continue;
            }
            rev_map.insert(namespace, alias);
        }
        while let Some(alias) = to_remove.pop() {
            println!("Dealiaser: rm service w/ mutiple aliases: {}", alias);
            map_w.remove(&alias);
        }
    }
}
