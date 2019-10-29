use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use std::cell::{RefMut, RefCell};

use crate::php::{Php,Class};

impl Php {
	pub fn rm_get(&mut self, work_dir: &str) {
		// let mut php_handle = self.classes.write().unwrap(); //.get_mut().unwrap();
		// let mut iter = php_handle.get_mut();

		// for (c_name, class) in php_handle.into_iter() {
		// 	println!("Class `{}`:\n{:?}\n\n\n", c_name, class);
		// }
	}
}
