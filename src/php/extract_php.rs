use regex::Regex;
use std::fs::File;
use std::io::prelude::*;

use crate::php::resolve_namespace::resolve_namespace;
use crate::php::{Class, Php};
use crate::php::{RE_CLASS, RE_CONSTRUCT, RE_GET, RE_NAMESPACE, RE_NO_CONSTRUCT, RE_USE};

impl Php {
    /// TODO
    // pub fn add_from_class_name(&mut self, full_name: &str, root_dir: &str) {
    // 	let path = full_name.split("\\");
    // 	let test = path.
    // }

    /// Returns class full name & Class
    fn extract_php(&self, php: &str, path: String) -> Option<(String, Class)> {
        let mut class = Class::new();

        if let Some(match_) = RE_CONSTRUCT.find(&php) {
            class.idx_construct_start = match_.start();
            class.has_constructor = true;
        } else if let Some(match_) = RE_NO_CONSTRUCT.find(&php) {
            class.idx_construct_start = match_.start();
            class.has_constructor = false;
        }
        if let Some(_) = RE_GET.find(&php) {
            class.has_get = true;
        }

        class.idx_construct_start = match RE_CONSTRUCT.find(&php) {
            Some(match_) => match_.start(),
            None => 0,
        };

        /* catch all `use` statements */
        for cap_use in RE_USE.captures_iter(&php) {
            let use_nspace = &cap_use[1];
            let use_name = match cap_use.get(3) {
                Some(alias) => alias.as_str(),
                None => {
                    let i = use_nspace.rfind("\\").unwrap_or(0);
                    &use_nspace[i..]
                }
            };
            class.idx_use_end = cap_use.get(0).unwrap().end();
            class
                .uses
                .insert(use_name.to_owned(), use_nspace.to_owned());
        }

        let class_nspace = {
            let n = match RE_NAMESPACE.captures(&php) {
                Some(c) => c,
                None => return None,
            }
            .get(1)
            .map_or("", |m| m.as_str())
            .to_owned();
            if n.ends_with("\\") {
                n
            } else {
                n + "\\"
            }
        };

        let (class_full_name, class_parent_full_name) = {
            let caps = match RE_CLASS.captures(&php) {
                Some(c) => c,
                None => return None, // not even a class name?
            };
            /* get short names from regex */
            let class_sname = caps.get(2).map_or("", |m| m.as_str());
            let parent_sname = caps.get(4).map_or("", |m| m.as_str());
            /* convert short names to full name (w/ namespace) */
            let class_nspace = format!("{}{}", class_nspace, class_sname);
            let parent_nspace = match class.uses.get(parent_sname) {
                Some(s) => Some(s.clone()),
                None => {
                    if parent_sname.is_empty() {
                        // No parent
                        None
                    } else {
                        Some(parent_sname.to_owned()) // Unknown parent namespace
                    }
                }
            };
            (class_nspace, parent_nspace)
        };
        class.parent = class_parent_full_name;
        class.path = path.to_owned();
        return Some((class_full_name, class));
    }

    pub fn add_from_php(&mut self, file_path: &str) {
        let mut file = File::open(file_path).unwrap(); // check err
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap_or(0);

        let (class_full_name, class) = match self.extract_php(&contents, file_path.to_owned()) {
            Some((class_fname, class)) => (class_fname, class),
            None => return,
        };
        drop(contents);
        self.add_class(file_path, &class_full_name, class);
    }

    fn add_class(&mut self, file_path: &str, class_full_name: &str, class: Class) {
        let mut classes_handle = self.classes.write().unwrap();
        /* Set curent class as child of parent class, if necessary */
        if let Some(class_parent_fname) = &class.parent {
            // if has parent
            let parent = classes_handle.get_mut(class_parent_fname); // find parent in map
            if parent.is_none() {
                if let Some(parent_path) = resolve_namespace(class_parent_fname) {
                    // resolve parent & add it
                    // println!("Recursion! Class {:100} Parent {}", class_full_name, class_parent_fname);
                    drop(classes_handle);
                    self.add_from_php(&parent_path);
                    let classes_handle = self.classes.read().unwrap();
                    if classes_handle.get(class_parent_fname).is_some() {
                        drop(classes_handle);
                        self.add_class(file_path, class_full_name, class);
                    }
                    return;
                } else {
                    eprintln!("Class not found `{}` !", class_parent_fname);
                }
            } else if let Some(c) = classes_handle.get_mut(class_parent_fname) {
                // bad
                c.children.push(class_full_name.to_owned());
            }
        }
        classes_handle.insert(class_full_name.to_owned(), class);
        drop(classes_handle);
    }
}