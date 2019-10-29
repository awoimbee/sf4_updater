use regex::Regex;
use std::fs::File;
use std::io::prelude::*;

use crate::php::{Php,Class};

impl Php {
    pub fn add_from_php(&mut self, file_path: &str) {
        lazy_static! {
            static ref RE_CLASS: Regex = // .get(1): class; .get(3): parent;
                Regex::new(r"\nclass ([^ \n]*)( extends ([^ \n]*))?").unwrap();
            static ref RE_NAMESPACE: Regex = // .get(1): namespace;
                Regex::new(r"\nnamespace ([^ ;]*);\n").unwrap();
            static ref RE_USE: Regex = // .get(1): real_class_name;  .get(3): as_alias;
                Regex::new(r"\nuse ([^ ;]*)( as ([^ ;]*))?;").unwrap();
        }

        let mut file = File::open(file_path).unwrap(); // check err
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap_or(0);

        let mut class = Class::new();
        /* catch all `use` statements */
        for cap_use in RE_USE.captures_iter(&contents) {
            let use_nspace = &cap_use[1];
            let use_name = match cap_use.get(3) {
                Some(alias) => alias.as_str(),
                None => {
                    let i = use_nspace.rfind("\\").unwrap_or(0);
                    &use_nspace[i..]
                }
            };
            class.uses.insert(use_name.to_owned(), use_nspace.to_owned());
        }

        let namespace = {
            let n = match RE_NAMESPACE.captures(&contents) {
                Some(c) => c,
                None => return
            }.get(1).map_or("", |m| m.as_str()).to_owned();
            if n.ends_with("\\") { n }
            else { n + "\\" }
        };

        let (class_full_name, class_parent_full_name) = {
            let caps = match RE_CLASS.captures(&contents) {
                Some(c) => c,
                None => return // not even a class name?
            };
            /* get short names from regex */
            let class_sname = caps.get(1).map_or("", |m| m.as_str());
            let parent_sname = caps.get(3).map_or("", |m| m.as_str());
            /* convert short names to full name (w/ namespace) */
            let class_nspace = format!("{}{}", namespace, class_sname);
            let parent_nspace = match class.uses.get(parent_sname) { // get parent from `use`
                Some(s) => Some(s.clone()),
                None => {
                    if parent_sname.is_empty() { None } // No parent
                    else { Some(parent_sname.to_owned()) } // Unknown parent namespace
                }
            };
            (class_nspace, parent_nspace)
        };
        class.parent = class_parent_full_name;
		class.path = file_path.to_owned();
		/* Get write handle on the whole Php struct */
        let mut classes_handle = self.classes.write().unwrap();
        /* Set curent class as child of parent class, if necessary */
        if let Some(c_p_f_n) = &class.parent {
            if let Some(c) = classes_handle.get_mut(c_p_f_n) {
                c.children.push(class_full_name.clone());
            }
        }

        classes_handle.insert(class_full_name, class);
        drop(classes_handle);
    }
}
