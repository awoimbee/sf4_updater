// use crate::php::Php;
// // use crate::php::resolve_namespace::*;
// // use crate::php::transformers::FileTransformer;
// // use colored::*;
// use super::super::FileTransformer;
// use super::SHELL_COMMANDS;
use super::FileMover;
// use crate::f_find::f_find;
use crate::G;
use regex::Regex;
use std::path::Path;

use std::fs::create_dir_all;

/// Returns new path
pub fn update_view_path(mut path: &str, fm: &mut FileMover) -> Result<String, &'static str> {
	lazy_static!{
		static ref PATH: Regex = Regex::new(r".*/(?P<bundle>[^/]*?)Bundle/Resources/views/?(?P<rel_path>.*)[/\\](?P<file>.*)").unwrap();
	}

	if !Path::new(path).exists() {
		let dir_path = &path[..path.rfind('/').unwrap_or(0)];
		if !Path::new(dir_path).exists() {
			return Err("File doesn't exist");
		}
		path = dir_path;
		println!("Yolo swag: {}", dir_path);
	}

	let pcap = PATH.captures(path).unwrap();
	let bundle = pcap.name("bundle").unwrap().as_str();
	let rel_path = pcap.name("rel_path").unwrap().as_str();
	let file = pcap.name("file").unwrap().as_str();
	let rel_sf_dirs = "app/Resources/views/";

	let new_path = format!(
		"{}{}{}/{}/{}",
		G.project_root,
		rel_sf_dirs,
		bundle,
		rel_path,
		file
	);

	let path_rel = &path[G.project_root.len()..];
	let new_path_rel = &new_path[G.project_root.len()..];
	let new_path_sf = &new_path_rel[rel_sf_dirs.len()..];
	let subdirs = &new_path[..new_path.len() - file.len()];

	if !fm.contains_dst(&new_path) {
		create_dir_all(Path::new(subdirs)).unwrap();
		fm.insert(path_rel.to_owned(), new_path_rel.to_owned());
	}

	Ok(new_path_sf.to_owned())
}
