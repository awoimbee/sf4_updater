use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

const POISONED: &str = "R O T T E N";

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

    // pub fn add(&mut self, class: &str, alias: &str) {
    //     let mut c = self.classes.write().unwrap();
    //     c.insert(alias.to_owned(), class.to_owned());
    // }

    pub fn dealias(&self, alias: &str) -> Option<String> {
        let c = self.classes.read().unwrap();
        let s = c.get(alias);
        if s.is_some() && !s.unwrap().starts_with(POISONED) {
            return Some(s.unwrap().to_owned());
        }
        return None;
    }

    pub fn add_from_yml(&mut self, file_path: &str) {
        // println!("Add from yml conf from: {}", file_path);
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
                    continue;
                    // s_alias
                } else {
                    continue;
                }
            };
            let (namespaced, pointed) = match s_name.contains("\\") {
                true => (s_name, s_alias),
                false => (s_alias, s_name),
            };
            // println!("\tadd {:50} => {}", pointed, namespaced);
            // A service w/ multiple aliases may use diff. args., straight autowiring is dangerous
            alias_map.insert(pointed.to_owned(), namespaced.to_owned());
        }
        drop(alias_map);
    }

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
            println!("dealiaser rm {}", alias);
            map_w.remove(&alias);
        }



        // unsafe {
        //     let mut to_remove: Vec<*const str> = Vec::new();
        //     for (alias, namespace) in map_w.iter() {
        //         if rev_map.contains_key(namespace.as_str()) {
        //             to_remove.push(rev_map.get(namespace.as_str()).unwrap().as_ref());
        //         }
        //         if rev_map.insert(namespace, alias).is_some() {
        //             to_remove.push(alias.as_str());
        //         }
        //     }
        //     while let Some(alias) = to_remove.pop() {
        //         println!("dealiaser rm {}", alias.as_ref().unwrap());
        //         map_w.remove(alias.as_ref().unwrap());
        //     }
        // }

    }
}
