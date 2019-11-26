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
pub fn update_controller_path(path: &str, fm: &mut FileMover) -> Result<String, &'static str> {
    // un 'Controller' a ninguna 'Action' es posible pero son pajero

    let sep_i = path.rfind(':').unwrap_or(path.len());
    let action = match path.get(sep_i..) {
        Some(a) => format!(":{}Action", a),
        None => String::new(),
    };
    let path = &path[..sep_i];

    if !Path::new(path).exists() {
        return Err("Controller doesn't exist");
    }

    let ft = match FileTransformer::new(path) {
        Some(ft) => ft,
        None => return Err("Could not open file"),
    };
    let nspace_cap = RE_NAMESPACE.captures(ft.reader()).unwrap(); // /!\ danger
    let classn_cap = RE_CLASS.captures(ft.reader()).unwrap();

    let new_path = format!(
        "{}\\{}{}",
        nspace_cap.get(1).unwrap().as_str(),
        classn_cap.name("name").unwrap().as_str(),
        action,
    );

    Ok(new_path)
}
