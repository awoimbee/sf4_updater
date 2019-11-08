use crate::php;
use crate::php::Class;
use std::collections::BTreeMap;
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
            contents,
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
        match self.read_ofst < self.contents.len() {
            true => &self.contents[self.read_ofst..],
            _ => &self.contents[self.contents.len() - 1..],
        }
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
    fn rewrite_uses(&mut self, class: &Class) {
        let uses_cap = php::RE_ALL_USE.find(&self.contents).unwrap(); // wil break
        let use_start = uses_cap.start();
        let uses_end = uses_cap.end();

        let mut sorted_uses: Vec<_> = class.uses.iter().collect();
        sorted_uses.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

        let new_uses: String = sorted_uses
            .iter()
            .map(|(k, v)| match v.ends_with(k.as_str()) {
                true => format!("\nuse {};", v),
                false => format!("\nuse {} as {};", v, k),
            })
            .collect::<String>();
        self.contents = format!(
            "{}{}{}",
            &self.contents[..use_start],
            new_uses,
            &self.contents[uses_end..]
        )
    }

    fn class_contains_var(&self, var: &str) -> bool {
        if self.contents.contains(&format!("    private ${};\n", var))
            || self
                .contents
                .contains(&format!("    protected ${};\n", var))
            || self.contents.contains(&format!("    public ${};\n", var))
        {
            true
        } else {
            false
        }
    }

    fn new_constructor(&mut self, args: &BTreeMap<String, String>) {
        if args.len() == 0 {
            return;
        }
        let where_new_c = php::RE_METH_N_DOC.find(&self.contents).unwrap().start();

        let mut new_construct_args = String::new();
        let mut new_construct_lines = String::new();
        let mut new_class_vars = String::new();
        for (typ, name) in args {
            new_construct_args.push_str(&format!("{} ${}, ", typ, name));
            new_construct_lines.push_str(&format!("$this->{0} = ${0};\n", name));
            if !self.class_contains_var(name) {
                new_class_vars.push_str(&format!("protected ${};\n", name));
            }
        }
        let new_construct_args = &new_construct_args[..new_construct_args.len() - 2];
        self.contents = format!(
            "{}\n{}\npublic function __construct({}) {{\n{}\n}}\n{}",
            &self.contents[..where_new_c],
            new_class_vars,
            new_construct_args,
            new_construct_lines,
            &self.contents[where_new_c..]
        );
    }

    fn update_constructor(&mut self, args: &BTreeMap<String, String>) {
        if args.len() == 0 {
            return;
        }
        let construct_cap = php::RE_CONSTRUCT.captures(&self.contents).unwrap();
        let where_new_c = construct_cap.get(0).unwrap().start();
        let args_cap = construct_cap.name("args").unwrap();
        let existing_args = &self.contents[args_cap.start()..args_cap.end()];

        let mut new_construct_args = String::new();
        let mut new_construct_lines = String::new();
        let mut new_class_vars = String::new();
        for (typ, name) in args {
            if existing_args.contains(&format!("${}", name)) {
                // FAILURE POINT
                continue;
            }
            new_construct_args.push_str(&format!("{} ${}, ", typ, name));
            new_construct_lines.push_str(&format!("$this->{0} = ${0};\n", name));
            if !self.class_contains_var(name) {
                new_class_vars.push_str(&format!("protected ${};\n", name));
            }
        }

        let where_args = construct_cap.name("args").unwrap().start();
        let where_body = construct_cap.get(0).unwrap().end();

        self.contents = format!(
            "{}\n{}\n{}{}{}\n{}{}",
            &self.contents[..where_new_c],
            new_class_vars,
            &self.contents[where_new_c..where_args],
            new_construct_args,
            &self.contents[where_args..where_body],
            new_construct_lines,
            &self.contents[where_body..],
        );
    }

    /// ##Add Args to constructor
    /// &Vec<(String, String)> -> &Vec<VarType, VarName>
    pub fn add_to_constructor(&mut self, args: &BTreeMap<String, String>) {
        if args.len() == 0 {
            return;
        }
        if php::RE_CONSTRUCT.find(&self.contents).is_some() {
            self.update_constructor(args);
        } else {
            self.new_constructor(args);
        }
    }
}
