use regex::Regex;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use crate::dealiaser::Dealiaser;
use crate::php::*;

fn add_use(class_content: &str, where_: usize, use_class: &str) -> String {
    String::new()
}

impl Php {
    pub fn rm_get(&mut self, dealiaser: &Dealiaser) {
        println!("rm_get");
        let pile_reader = self.has_get_stack.read().unwrap();

        for class_name in pile_reader.iter() {
            println!("\tName {}", class_name);
            let classes_r = self.classes.read().unwrap();
            let class_mutex = classes_r.get(class_name).unwrap().clone();
            drop(classes_r);
            let class = class_mutex.lock().unwrap();

            if !class.has_constructor
                && class.parent.is_some()
            {
                let parent_class_name = class.parent.as_ref().unwrap();
                if self.load_class(parent_class_name).is_none() {
                    println!("\t\tCannot load parent class `{}`", parent_class_name);
                    continue;
                }
                if self.classes.read().unwrap().get(parent_class_name).unwrap().lock().unwrap().has_constructor {
                    println!("\t\tCannot update constructors from parent & shit right now");
                    continue;
                }
            }

            let mut file = File::open(&class.path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap_or(0);

            while let Some(get_cap) = RE_GET.captures(&contents) {
                let get_alias = &get_cap[1];
                let get_namespaced = match dealiaser.dealias(get_alias) {
                    Some(nspace) => nspace,
                    None => break, // println!("Could not dealias {} !", get_alias);
                };
                println!("\t{:50} => {}", get_alias, get_namespaced);
                break;
                // let new_contents = add_use(&contents, class.idx_use_end, &get_namespaced);
                // for cap_use in RE_USE.captures_iter(&contents) {
            }

        }
    }
}
