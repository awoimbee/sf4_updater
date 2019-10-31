use crate::php::Class;
use crate::php::RE_ALL_USE;
use std::fs::OpenOptions;
use std::io::prelude::*;

mod dealias_get_repository;
mod rm_get;

fn rewrite_uses(file_contents: String, class: &Class) -> String {
    let uses_cap = RE_ALL_USE.find(&file_contents).unwrap();
    let use_start = uses_cap.start();
    let uses_end = uses_cap.end();

    let new_uses: String = class
        .uses
        .iter()
        .map(|(k, v)| match v.ends_with(k) {
            true => format!("\nuse {};", v),
            false => format!("\nuse {} as {};", v, k),
        })
        .collect::<String>();

    format!(
        "{}{}{}",
        &file_contents[..use_start],
        new_uses,
        &file_contents[uses_end..]
    )
}

fn write_file(file_contents: &str, file_name: &str) -> bool {
    let open_options = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_name);
    let mut file_w = match open_options {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not open file ({})", e);
            return false;
        }
    };
    match file_w.write(file_contents.as_bytes()) {
        Ok(_size) => true,
        Err(e) => {
            eprintln!("Could write to file ({})", e);
            false
        }
    }
}
