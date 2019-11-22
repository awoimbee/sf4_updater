// use crate::php::Php;
// // use crate::php::resolve_namespace::*;
// // use crate::php::transformers::FileTransformer;
// // use colored::*;
// use super::super::FileTransformer;
use super::SHELL_COMMANDS;
// use crate::f_find::f_find;
use crate::G;
use regex::Regex;
use std::path::Path;

use std::fs::create_dir_all;


// pub fn update_twig_paths() {
//     println!("\tupdate_twig_paths");
//     f_find(&G.work_dir, r".*\.twig", |s| dealias_path(s));
// }

// Does the file exist in the specified directory ?
// Did the file already get moved and exists in the new directory ?
// -> move the file ?

/// Returns new path
pub fn update_view_path(path: &str) -> Result<String, &'static str> {
	lazy_static!{
		static ref PATH: Regex = Regex::new(r".*/(?P<bundle>[^/]*?)Bundle/Resources/views/?(?P<rel_path>.*)/(?P<file>.*)").unwrap();
	}
	let pcap = PATH.captures(path).unwrap();
	// let subdirs_rel = pcap.name("rel_path").unwrap().as_str();
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
	println!("=> {}", new_path);

	let path_rel = &path[G.project_root.len()..];
	let new_path_rel = &new_path[G.project_root.len()..];
	let new_path_sf = &new_path_rel[rel_sf_dirs.len()..];
	let subdirs = &new_path[..new_path.len() - file.len()];

	let old_path_exists = Path::new(path).exists();
	let new_path_exists = Path::new(&new_path).exists();

	// println!("{} -> {}", path, new_path);

	match (old_path_exists, new_path_exists) {
		(false, true) => (),
		(false, false) => return Err("File doesn't exist"),
		(true, true) => return Err("Name conflict"),
		(true, false) => move_file(path_rel, new_path_rel, subdirs),
	};

	Ok(new_path_sf.to_owned())
}

fn move_file(old_path: &str, new_path: &str, subdirs: &str) {
	create_dir_all(Path::new(subdirs)).unwrap();
	let git_mv_cmd = vec!["git".to_owned(), "-C".to_owned(), G.project_root.clone(), "mv".to_owned(), old_path.to_owned(), new_path.to_owned()];
	SHELL_COMMANDS.lock().unwrap().push(git_mv_cmd);
}
