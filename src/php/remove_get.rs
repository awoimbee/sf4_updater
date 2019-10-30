use regex::Regex;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use crate::dealiaser::Dealiaser;
use crate::php::*;

impl Php {
    pub fn rm_get(&mut self, dealiaser: &Dealiaser, work_dir: &str) {
        // println!("{:?}", dealiaser);
        let classes = self.classes.clone();
        let php_handle = classes.read().unwrap(); //.get_mut().unwrap();

        for (c_name, class) in php_handle.iter() {
            if !class.path.starts_with(work_dir) || !class.has_get {
                // println!("x '{}' != '{}'", work_dir, c_name);
                continue;
            }

            let test = &class.parent;
            let pd = test.unwrap();
            if class.has_constructor
            && class.parent.is_some()
            && self.load_class(&class.parent.unwrap()).is_some() {

            }
            let mut file = File::open(&class.path).unwrap(); // check err
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap_or(0);

            // println!("Class `{}`: {:?}", c_name, class);

            for get in RE_GET.captures_iter(&contents) {
                let get_alias = &get[1];
                // println!("{}", get_alias);

                let get_namespaced = match dealiaser.dealias(get_alias) {
                    Some(nspace) => nspace,
                    None => {
                        println!("Could not dealias {} !", get_alias);
                        continue;
                    }
                };




                println!("Class `{}`: {}", get_alias, get_namespaced);
                // for cap_use in RE_USE.captures_iter(&contents) {
            }
        }
    }

}
