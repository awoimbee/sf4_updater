use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

use crate::php;
use crate::php::resolve_namespace::resolve_namespace;
use crate::php::{Class, Php};

impl Php {
    /// TODO
    // pub fn add_from_class_name(&mut self, full_name: &str, root_dir: &str) {
    // 	let path = full_name.split("\\");
    // 	let test = path.
    // }

    /// Returns class full name & Class
    fn extract_php(&self, php: &str, path: String) -> Option<(String, Class)> {
        let mut class = Class::new();

        if let Some(_match) = php::RE_CONSTRUCT.find(&php) {
            class.has_constructor = true;
        }
        if let Some(_) = php::RE_GET.find(&php) {
            class.has_get = true;
        }
        if let Some(_) = php::RE_GETREPOSITORY.find(&php) {
            class.has_get_repository = true;
        }

        /* catch all `use` statements */
        for cap_use in php::RE_USE.captures_iter(&php) {
            let use_nspace = &cap_use[1];
            let use_name = match cap_use.get(3) {
                Some(alias) => alias.as_str(),
                None => php::class_name(&use_nspace),
            };
            class
                .uses
                .insert(use_name.to_owned(), use_nspace.to_owned());
        }

        let class_nspace = match php::RE_NAMESPACE.captures(&php) {
            Some(cap) => {
                let cnspace = &cap[1];
                match cnspace.ends_with("\\") {
                    true => format!("{}", cnspace),
                    false => format!("{}\\", cnspace),
                }
            }
            None => return None,
        };

        let (class_full_name, class_parent_full_name) = {
            let caps = match php::RE_CLASS.captures(&php) {
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

    /// /!\ Write lock on classes
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
        let parent_name = &class.parent.clone().unwrap();
        let some_parent = classes_r.get(parent_name);

        if some_parent.is_some() {
            let mut class = some_parent.unwrap().lock().unwrap();
            class.children.push(class_full_name.to_owned());
            return;
        } else if let Some(parent_path) = resolve_namespace(parent_name) {
            drop(classes_r);
            self.add_from_php(&parent_path);
            let succesful_add = {
                let classes_r = self.classes.read().unwrap();
                classes_r.get(parent_name).is_some()
            };
            if succesful_add {
                self.set_parent(file_path, class_full_name, class); // retry add
            }
        }
    }

    /// /!\ Write lock on classes & has_*_stack
    fn add_class(&self, file_path: &str, class_full_name: &str, class: Class) {
        let has_get = class.has_get;
        let has_get_repository = class.has_get_repository;

        /* Set curent class as child of parent class, if necessary */
        if let Some(_) = &class.parent {
            self.set_parent(file_path, class_full_name, &class);
        }
        /* insert class */
        let mut classes_w = self.classes.write().unwrap();
        classes_w.insert(class_full_name.to_owned(), Arc::new(Mutex::new(class)));
        drop(classes_w);
        /* insert class in workstack if necessary */
        let work_dir: &str = &crate::WORK_DIR.read().unwrap();
        if file_path.starts_with(work_dir) {
            if has_get {
                let mut workstack_w = self.has_get_stack.write().unwrap();
                workstack_w.push(class_full_name.to_owned());
            }
            if has_get_repository {
                let mut workstack_w = self.has_get_repository_stack.write().unwrap();
                workstack_w.push(class_full_name.to_owned());
            }
        }
        // println!("class added {}", class_full_name);
    }
}
