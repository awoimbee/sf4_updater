// use crate::php::Php;
// // use crate::php::resolve_namespace::*;
// // use crate::php::transformers::FileTransformer;
// // use colored::*;
// use super::super::FileTransformer;
// use super::SHELL_COMMANDS;
use super::FileMover;
use crate::f_find::f_find;
use crate::G;
use regex::Regex;
use std::path::Path;

use std::fs::create_dir_all;

/// Returns new path
pub fn update_view_path(mut path: &str, fm: &mut FileMover) -> Result<String, &'static str> {
    lazy_static! {
        static ref PATH: Regex = Regex::new(
            r".*/(?P<bundle>[^/]*?)Bundle/Resources/views/?(?P<rel_path>.*)[/\\](?P<file>.*)"
        )
        .unwrap();
    }

    if !Path::new(path).exists() {
        let par = &path[..path.rfind('/').unwrap_or(0)];
        if !Path::new(par).exists() {
            return Err("File & parent folder doesn't exist");
        }
        eprintln!("File {} doesn't exist, using parent dir", par);
        path = par;
    }

    let is_dir = match std::fs::metadata(Path::new(path)) {
        Ok(m) if m.is_dir() => true,
        Ok(m) if !m.is_dir() => false,
        Ok(_) => return Err(""), // useless
        Err(_) => return Err("Could not get file metadata"),
    };

    if is_dir {
        f_find(path, r".*\.twig", |p| {
            drop(super::update_path(p, "", fm));
        }); // mhhh
    }

    let pcap = PATH.captures(path).unwrap();
    let bundle = pcap.name("bundle").unwrap().as_str();
    let rel_path = pcap.name("rel_path").unwrap().as_str();
    let file = pcap.name("file").unwrap().as_str();
    let rel_sf_dirs = "templates/";

    let new_path_part = format!("{}/{}/{}", bundle, rel_path, file); //.to_ascii_lowercase();
    let new_path = format!("{}{}{}", G.project_root, rel_sf_dirs, new_path_part);

    let path_rel = &path[G.project_root.len()..];
    let new_path_rel = &new_path[G.project_root.len()..];
    let new_path_sf = &new_path_rel[rel_sf_dirs.len()..];
    let subdirs = &new_path[..new_path.len() - file.len()];

    if !is_dir && !fm.contains_dst(&new_path) {
        create_dir_all(Path::new(subdirs)).unwrap();
        fm.insert(new_path_rel.to_owned(), path_rel.to_owned());
    }

    Ok(new_path_sf.to_owned())
}
