use regex::Regex;
use std::fs::File;
use std::io::prelude::*;

use crate::php::resolve_namespace::resolve_namespace;
use crate::php::{Class, Php};
use crate::php::{RE_CLASS, RE_CONSTRUCT, RE_GET, RE_NAMESPACE, RE_NO_CONSTRUCT, RE_USE, RE_GETREPOSITORY};

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
            // class.idx_construct_start = match_.start();
            class.has_constructor = true;
        } else if let Some(match_) = RE_NO_CONSTRUCT.find(&php) {
            // class.idx_construct_start = match_.start();
            class.has_constructor = false;
        }
        if let Some(_) = RE_GET.find(&php) {
            // println!("{} has get", path);
            class.has_get = true;
        }
        if let Some(_) = RE_GETREPOSITORY.find(&php) {
            // println!("{} has get", path);
            class.has_get_repository = true;
        }


        // class.idx_construct_start = match RE_CONSTRUCT.find(&php) {
        //     Some(match_) => match_.start(),
        //     None => 0,
        // };

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
            // class.idx_use_end = cap_use.get(0).unwrap().end();
            class.uses.insert(use_name.to_owned(), use_nspace.to_owned());
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

    pub fn add_from_php(&self, file_path: &str) {
        let mut file = File::open(file_path).unwrap(); // check err
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap_or(0);

        let (class_full_name, class) = match self.extract_php(&contents, file_path.to_owned()) {
            Some((class_fname, class)) => (class_fname, class),
            None => return,
        };
        self.add_class(file_path, &class_full_name, class);
    }

    fn set_parent(&self, file_path: &str, class_full_name: &str, class: &Class) {
        let classes_r = self.classes.read().unwrap();
        let parent_name = &class.parent.clone().unwrap(); // parent_name
        let known_parent = classes_r.get(parent_name).is_some();
        drop(classes_r);

        if known_parent {
            let mut classes_w = self.classes.write().unwrap();
            classes_w.get_mut(parent_name).unwrap().children.push(class_full_name.to_owned());
            drop(classes_w);
            return;
        }

        if let Some(parent_path) = resolve_namespace(parent_name) {
            self.add_from_php(&parent_path);
            // let classes_r = self.classes.read().unwrap();
            let succesful_add = {
                let classes_r = self.classes.read().unwrap();
                classes_r.get(parent_name).is_some()
            };
            if succesful_add { // if add_from_php was succesful
                self.set_parent(file_path, class_full_name, class); // retry add
            }
            return;
        }

    }

    fn add_class(&self, file_path: &str, class_full_name: &str, class: Class) {
        let has_get = class.has_get;
        let has_get_repository = class.has_get_repository;
        /* Set curent class as child of parent class, if necessary */
        if let Some(_) = &class.parent {
            self.set_parent(file_path, class_full_name, &class);
        }
        let mut classes_w = self.classes.write().unwrap();
        classes_w.insert(class_full_name.to_owned(), class);
        drop(classes_w);

        let work_dir: &str = &crate::WORK_DIR.read().unwrap();
        if (has_get || has_get_repository) && file_path.to_owned().starts_with(work_dir) {
            // println!("PUSH to workstack");
            let mut workstack_w = self.work_stack.write().unwrap();
            workstack_w.push(class_full_name.to_owned());
        }
        // println!("class added {}", class_full_name);
    }
}
