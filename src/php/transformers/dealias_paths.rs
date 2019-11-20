use crate::php;
// use crate::php::resolve_namespace::*;
// use crate::php::transformers::FileTransformer;
// use colored::*;
use crate::G;
use crate::f_find::f_find;
use regex::Regex;
use super::FileTransformer;


fn colon_to_path(colon_alias: regex::Captures<'_>) {

}

fn dealias_path(path: &str) {
	/* Build regex */
	let re_path_alias = {
		let mut bundles_reg_str = G.bundles.iter().map(|(n, _p)| format!("{}|", n)).collect::<String>();
		bundles_reg_str.pop();
		let reg_str = format!(
			concat!(
				r"(?P<colon>{root_nspace}(?P<colon_bundle>{bundles})Bundle:(?P<colon_path>[^: ]*):(?P<colon_file>[^'\s ]*))",
				r"|(?P<shortBundle>@{root_nspace}(?P<shortBundle_bundle>{bundles})Bundle/(?P<shortBundle_path>[^'\s]*))",
				r"|(?P<short>@{root_nspace}(?P<short_bundle>{bundles})/(?P<short_path>[^'\s]*))",
			),
			bundles = bundles_reg_str,
			root_nspace = G.root_namespace
		);
		match Regex::new(&reg_str) {
			Ok(r) => r,
			Err(e) => {
				println!("Bad bundle name (could not build regex from it) ({}): {}", reg_str, e);
				return;
			}
		}
	};
	/* ########### */

	let mut ft = FileTransformer::new(&path);

	while let Some(path_cap) = re_path_alias.captures(ft.reader()) {
		// colon goes to path_to_bundle/Sesources/views/
		// short goes to path_to_bundle/Sesources/views/
		// shortBundle goes to path_to_bundle
		// let t = path_cap.name("colon").unwrap();

		match () {
			_ if path_cap.name("colon").is_some() => println!("Case colon: '{}'", &path_cap[0]),
			_ if path_cap.name("shortBundle").is_some() => println!("Case shortBundle: '{}'", &path_cap[0]),
			_ if path_cap.name("short").is_some() => println!("Case short: '{}'", &path_cap[0]),
			_ => {
				ft.reader_skip(path_cap.get(0).unwrap().end());
				continue;
			}
		}
		ft.reader_skip(path_cap.get(0).unwrap().end());

		// let full_match = get_cap.get(0).unwrap();
		// let alias_match = get_cap.get(1).unwrap();
		// let fmatch_bounds = (full_match.start(), full_match.end());

	}
}

impl php::Php {
    pub fn dealias_paths(&mut self) {
		println!("dealias_paths");
		// fuck it, not using a work stack

		f_find(&G.work_dir, ".php", |s| dealias_path(s));


		// let zero = |php, x| {
		// 	println!("test {:?}", x);
		// };

		// crate::f_find(&crate::G.work_dir, "", |x| zero(self, x));


		// // zero(&self);

    }
}
