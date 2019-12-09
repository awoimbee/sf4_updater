// use crate::php::Php;
// // use crate::php::resolve_namespace::*;
// // use crate::php::transformers::FileTransformer;
// // use colored::*;
// use super::super::FileTransformer;
// use super::SHELL_COMMANDS;

use super::super::php::RE_CLASS;
use super::super::php::RE_NAMESPACE;
use super::super::FileTransformer;
use super::FileMover;
// use crate::f_find::f_find;
use crate::G;
use regex::Regex;
use std::path::Path;

use std::fs::create_dir_all;

/// Returns new path
pub fn update_controller_path(fpath: &str, context: &str) -> Result<String, &'static str> {
    // un 'Controller' a ninguna 'Action' es posible pero son pajero

    let sep_i = fpath.rfind(':').unwrap_or(fpath.len());
    let path = &fpath[..sep_i];

    if !Path::new(path).exists() {
        return Err("Controller doesn't exist");
    }
    let ft = match FileTransformer::new(path) {
        Some(ft) => ft,
        None => return Err("Could not open file"),
    };
    let r = ft.reader();


    let action = match fpath.get(sep_i + 1..) {
        Some(a) => {
                if let Some(_) = r.find(&format!("public function {}(", a)) {
                    format!("::{}", a)
                } else {
                    let a = format!("{}Action", a);
                    if let Some(_) = r.find(&format!("public function {}(", a)) {
                        format!("::{}", a)
                    } else {
                        return Err("Method doesn't exist !");
                    }
                }
            },
        None => String::new(),
    };


    let nspace_cap = RE_NAMESPACE.captures(ft.reader()).unwrap(); // /!\ danger
    let classn_cap = RE_CLASS.captures(ft.reader()).unwrap();

    let mut new_path = format!(
        "{}\\{}{}",
        nspace_cap.get(1).unwrap().as_str(),
        classn_cap.name("name").unwrap().as_str(),
        action,
    );
    if context.ends_with(".twig") {
        let mut tmp = String::new();
        for c in new_path.bytes() {
            match c as char {
                '\\' => tmp.push_str("\\\\"),
                _ => tmp.push(c as char),
            };
        }
        new_path = tmp;
    }
    Ok(new_path)
}
