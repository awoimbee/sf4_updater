// use std::collections::HashMap;

// use crate::php::RE_CONSTRUCT;

// struct Arg<'a> {
// 	name: &'a str,
// 	type_: &'a str,
// 	def_val: &'a str
// }

// fn get_constructor(php: &str) -> Option<&str> {
// 	let cap = match RE_CONSTRUCT.find(php) {
// 		Some(c) => c,
// 		None => return None
// 	};
// 	Some(cap.as_str())
// }

// fn get_constructor_args<'a>(php: &'a str) -> Option<&'a str> {
// 	let cap = match RE_CONSTRUCT.captures(php) {
// 		Some(c) => c,
// 		None => return None
// 	};
// 	Some(&cap[1])
// }

// fn split_constructor_args(args: &str) -> Vec<Arg> {
// 	let mut res = Vec::new();
// 	let args_iter = args.split(',');
// 	for arg_match in args_iter {
// 		let mut var_name = "";
// 		let mut var_type = "";
// 		let mut var_def_val = "";
// 		// let mut arg_struct = Arg {"", "", ""};
// 		let arg_sep_type: Vec<&str> = arg_match.trim().split(&[' ', '='][..]).collect();
// 		match arg_sep_type.len() {
// 			1 => { // only var name
// 				var_name = arg_sep_type[0];
// 			}
// 			2 => { // type + var name
// 				var_type = arg_sep_type[0];
// 				var_name = arg_sep_type[1];
// 			}
// 			3 => {
// 				var_type = arg_sep_type[0];
// 				var_name = arg_sep_type[1];
// 				var_def_val = arg_sep_type[2];
// 			}
// 			_ => {
// 				eprintln!("Cannot parse arg: {}", arg_match);
// 				continue;
// 			}
// 		}
// 		let arg_class = Arg{
// 			name: var_name,
// 			type_: var_type,
// 			def_val: var_def_val
// 		};
// 		res.push(arg_class);
// 	}
// 	return res;
// }



