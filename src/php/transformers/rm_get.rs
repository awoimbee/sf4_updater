// use regex::Regex;
// use std::cell::{RefCell, RefMut};
// use std::collections::HashMap;
// use std::fs::File;
// use std::io::prelude::*;

// use crate::dealiaser::Dealiaser;
// use crate::php::*;

// fn add_use(class_content: &str, where_: usize , use_class: &str) -> String {
//     String::new()
// }

// impl Php {
//     pub fn rm_get(&mut self, dealiaser: &Dealiaser, work_dir: &str) {
//         let pile_reader = self.work_stack.read().unwrap();

//         for class_name in pile_reader.iter() {
//             println!("Name {}", class_name);

//             let classes_r = self.classes.read().unwrap();
//             let class = classes_r.get(class_name).unwrap().clone();
//             drop(classes_r);

//             if class.has_constructor
//                 && class.parent.is_some()
//                 && self.load_class(&class.parent.clone().unwrap()).is_some()
//             {
//                 println!("Cannot update constructors & shit right now");
//                 continue;
//             }

//             let mut file = File::open(&class.path).unwrap(); // check err
//             let mut contents = String::new();
//             file.read_to_string(&mut contents).unwrap_or(0);

//             let get_cap = RE_GET.captures_iter(&contents);

//             for get in get_cap {
//                 let get_alias = &get[1];
//                 let get_namespaced = match dealiaser.dealias(get_alias) {
//                     Some(nspace) => nspace,
//                     None => continue // println!("Could not dealias {} !", get_alias);
//                 };
//                 println!("\t{:50} => {}", get_alias, get_namespaced);
//                 // let new_contents = add_use(&contents, class.idx_use_end, &get_namespaced);
//                 // for cap_use in RE_USE.captures_iter(&contents) {
//             }
//         }
//     }
// }
