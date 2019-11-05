use crate::php::Class;
use crate::php;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

mod dealias_get_repository;
mod rm_get;

#[derive(Debug)]
struct FileTransformer {
    contents: String,
    read_ofst: usize,
}

impl FileTransformer {
    pub fn new(file_name: &str) -> Self {
        let mut contents = String::new();
        match File::open(file_name) {
            Err(e) => eprintln!("Could no open `{}` ({})", file_name, e),
            Ok(mut f) => drop(f.read_to_string(&mut contents).unwrap_or(0)),
        };
        FileTransformer {
            contents: contents,
            read_ofst: 0,
        }
    }
    pub fn reader_replace(&mut self, re_start: usize, re_end: usize, replacement: &str) {
        let before = re_start + self.read_ofst;
        let after = re_end + self.read_ofst;
        self.contents = format!(
            "{}{}{}",
            &self.contents[..before],
            replacement,
            &self.contents[after..]
        );
        self.read_ofst = after;
    }
    pub fn reader_skip(&mut self, cap_end: usize) {
        self.read_ofst = cap_end + 1 + self.read_ofst;
    }
    pub fn reader(&self) -> &str {
        &self.contents[self.read_ofst..]
    }
    pub fn get_mut(&mut self) -> &mut String {
        &mut self.contents
    }
    fn write_file(&self, file_name: &str) -> bool {
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
        match file_w.write(self.contents.as_bytes()) {
            Ok(_size) => true,
            Err(e) => {
                eprintln!("Could write to file ({})", e);
                false
            }
        }
    }
}

impl FileTransformer {
    fn rewrite_uses(&mut self, class: &Class) -> String {
        let uses_cap = php::RE_ALL_USE.find(&self.contents).unwrap();
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
            &self.contents[..use_start],
            new_uses,
            &self.contents[uses_end..]
        )
    }
    /// LOTS OF TODO HERE
    pub fn new_constructor_injection(&mut self, var_type: &str, var_name: &str) {
        println!("new_constructor_injection");
        let wher = php::RE_METH_N_DOC.find(&self.contents).unwrap().start(); // TODO: WILL EXPLODE
        let before = &self.contents[..wher];
        let after = &self.contents[wher..];
        let args = format!("{} ${}", var_type, var_name);
        let body = format!("$this->{0} = ${0};", var_name);
        // Big yolo

        self.contents = format!(
            "{}\nprivate ${};\npublic function __construct({}) {{\n{}\n}}\n{}",
            before,
            var_name,
            args,
            body,
            after
        );
        println!("DONE");
    }
    pub fn update_constructor_injection(&mut self, var_type: &str, var_name: &str) {
        println!("update_constructor_injection");
        let construct_match = php::RE_CONSTRUCT.find(&self.contents).unwrap();
        //TODO
    }
    pub fn add_class_var(&mut self, typeh: &str, var_name: &str) {
        let wher = php::RE_METH_N_DOC.find(&self.contents).unwrap().start(); // TODO: WILL EXPLODE
        let before = &self.contents[wher..];
        let after = &self.contents[..wher];

    }
}
