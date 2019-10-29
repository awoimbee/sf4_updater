use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

mod extract_php;
mod remove_get;
pub mod resolve_namespace;

lazy_static! {
    static ref RE_CLASS: Regex = // .get(2): class; .get(4): parent;
        Regex::new(r"\n(abstract )?class ([^ \n]*)( extends ([^ \n]*))?").unwrap();
    static ref RE_NAMESPACE: Regex = // .get(1): namespace;
        Regex::new(r"\nnamespace ([^ ;]*);\n").unwrap();
    static ref RE_USE: Regex = // .get(1): real_class_name;  .get(3): as_alias;
        Regex::new(r"\nuse ([^ ;]*)( as ([^ ;]*))?;").unwrap();
    static ref RE_CONSTRUCT: Regex =
        Regex::new(r"public function __construct\(").unwrap();
    static ref RE_NO_CONSTRUCT: Regex =
        Regex::new(r"\n[ \t]*.*?function [^ (]*").unwrap();
    static ref RE_GET: Regex = // .get(0): $this->get; .get(1): class
        Regex::new(r"\$this->get\('(.*?)'\)").unwrap();
}

#[derive(Debug)]
struct Class {
    path: String,
    children: Vec<String>,
    parent: Option<String>,
    uses: HashMap<String, String>, // Name \ Class
    idx_use_end: usize,
    idx_construct_start: usize,
    has_constructor: bool,
    has_get: bool,
}

#[derive(Debug, Clone)]
pub struct Php {
    classes: Arc<RwLock<HashMap<String, Class>>>,
}

impl Class {
    pub fn new() -> Class {
        let children = Vec::new();
        let parent = None;
        let uses = HashMap::new();
        let path = String::new();
        Class {
            path,
            children,
            parent,
            uses,
            idx_use_end: 0,
            idx_construct_start: 0,
            has_constructor: false,
            has_get: false,
        }
    }
}

impl Php {
    pub fn new() -> Php {
        let h_map: HashMap<String, Class> = HashMap::new();
        let c = Arc::new(RwLock::new(h_map));
        Php { classes: c }
    }

    // pub fn get_class<'a>(&mut self, alias: &str) -> &'a Class {

    // }
}
