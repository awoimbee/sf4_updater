use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

use crate::php;
use crate::php::resolve_namespace::namespace_to_path;
use crate::php::{Class, Php};

// pub interface
impl Php {
    /// /!\ Write lock on classes
    pub fn add_from_php(&self, file_path: &str) {
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap_or(0);

        let (class_full_name, class) = match self.extract_php(&contents, file_path.to_owned()) {
            Some((class_fname, class)) => (class_fname, class),
            None => return,
        };
        self.add_class(file_path, &class_full_name, class);
    }
}

impl Php {
    /// Returns class full name & Class
    fn extract_php(&self, php: &str, path: String) -> Option<(String, Class)> {
        let mut class = Class::new();

        /* Catch `use` statements */
        for cap_use in php::RE_USE.captures_iter(&php) {
            let use_nspace = cap_use.name("class").unwrap().as_str().to_owned();
            let use_name = match cap_use.name("alias") {
                Some(alias) => alias.as_str().to_owned(),
                None => php::class_name(&use_nspace).to_owned(),
            };
            class.uses.insert(use_name, use_nspace);
        }
        /* Catch namespace */
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
        /* Catch constructor args */
        if let Some(caps) = php::RE_CONSTRUCT.captures(&php) {
            lazy_static! {
                static ref RE_SEPARATE_ARGS: Regex =
                    Regex::new(r"^(\s*(?:(?:\??[a-zA-Z\-_0-9]*)\s*)?(?:&?\$[a-zA-Z\-_0-9]*)(?:\s*=\s*(?:.*?))?[,]?\s*)+$").unwrap();
                static ref RE_ARG_INFOS: Regex =
                    Regex::new(r"(?:(?P<type>[\\a-zA-Z\-_0-9]*) )?(?P<name>&?\$[a-zA-Z\-_0-9]*)(?:[ \t]*=[ \t]*(?P<def>.*))?").unwrap();
            }
            let mut args = caps.name("args").unwrap().as_str();

            while let Some(arg_cap) = RE_SEPARATE_ARGS.captures(args) {
                let arg_cap = arg_cap.get(1).unwrap();
                let arg_parts = RE_ARG_INFOS.captures(arg_cap.as_str()).unwrap();
                let name = arg_parts.name("name").unwrap().as_str();
                let name = name[name.find('$').unwrap() + 1..].to_owned();
                let def_val = match arg_parts.name("def") {
                    Some(def) => Some(def.as_str().to_owned()),
                    None => None,
                };
                let typeh = match arg_parts.name("type") {
                    Some(t) => match class.uses.get(t.as_str()) {
                        Some(f_t) => Some(f_t.clone()),
                        None => {
                            println!("No use found for {}", t.as_str());
                            Some(t.as_str().to_owned())
                        }
                    },
                    None => None,
                };
                let in_class_name_re =
                    Regex::new(&format!("(\\$this->[a-zA-Z\\-_0-9]*) = \\${};", name)).unwrap();
                let in_class_name = match in_class_name_re.captures(php) {
                    Some(m) => Some(m[1].to_owned()),
                    None => None,
                };
                let arg = php::Arg {
                    name,
                    typeh,
                    def_val,
                    in_class_name,
                };
                class.construct_args.push(arg);
                args = &args[..arg_cap.start()];
            }
        }
        if let Some(_) = php::RE_GET.find(&php) {
            class.has_get = true;
        }
        if let Some(_) = php::RE_GETREPOSITORY_ALIAS.find(&php) {
            class.has_get_repository = true;
        }

        let (class_full_name, class_parent_full_name) = {
            let caps = match php::RE_CLASS.captures(&php) {
                Some(c) => c,
                None => return None,
            };
            /* get short names from regex */
            let class_sname = caps.name("name").map_or("", |m| m.as_str());
            let class_nspace = format!("{}{}", class_nspace, class_sname);

            let parent_sname = caps.name("parent").map_or("", |m| m.as_str());
            let parent_nspace = match class.uses.get(parent_sname) {
                Some(s) => Some(s.clone()),
                None => {
                    if parent_sname.is_empty() {
                        None
                    } else {
                        // parent namespace not explicit, lets use the implicit namespace
                        let parent_full_name = format!("{}{}", class_nspace, parent_sname);
                        match namespace_to_path(&parent_full_name).is_some() {
                            true => Some(parent_full_name),
                            false => None,
                        }
                    }
                }
            };
            (class_nspace, parent_nspace)
        };
        class.parent = class_parent_full_name;
        class.path = path.to_owned();
        return Some((class_full_name, class));
    }

    fn set_parent(&self, file_path: &str, class_full_name: &str, class: &Class) {
        let classes_r = self.classes.read().unwrap();
        let parent_name = &class.parent.clone().unwrap();
        let some_parent = classes_r.get(parent_name);

        if some_parent.is_some() {
            let mut class = some_parent.unwrap().lock().unwrap();
            class.children.push(class_full_name.to_owned());
            return;
        } else if let Some(parent_path) = namespace_to_path(parent_name) {
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
        /* insert class in workstack, if necessary */
        let work_dir: &str = &crate::WORK_DIR.read().unwrap();
        if file_path.starts_with(work_dir) {
            if has_get {
                // let mut workstack_w = match self.has_get_stack.try_write() {
                //     Ok(writer) => writer,
                //     Err(_e) => { println!("get wtf is happening w/ {} ??!", file_path ); return; }
                // };
                let mut workstack_w = self.has_get_stack.write().unwrap();
                workstack_w.push(class_full_name.to_owned());
            }
            if has_get_repository {
                // let mut workstack_w = match self.has_get_repository_stack.try_write() {
                //     Ok(writer) => writer,
                //     Err(_e) => { println!("getrepo wtf is happening w/ {} ??!", file_path ); return; }
                // };
                let mut workstack_w = self.has_get_repository_stack.write().unwrap();
                workstack_w.push(class_full_name.to_owned());
            }
        }
        // println!("class added {}", class_full_name);
    }
}
