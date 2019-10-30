use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
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

#[derive(Clone)]
pub struct Php {
    classes_writer: Arc<Mutex<evmap::WriteHandle<String, Class>>>,
    classes_reader_factory: evmap::ReadHandleFactory<String, Class>,
    // classes: Arc<RwLock<HashMap<String, Class>>>,
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

impl Clone for Class {
    #[inline]
    fn clone(&self) -> Self {
        Class {
            path: self.path.clone(),
            children: self.children.clone(),
            parent: self.parent.clone(),
            uses: self.uses.clone(),
            idx_use_end: self.idx_use_end,
            idx_construct_start: self.idx_construct_start,
            has_constructor: self.has_constructor,
            has_get: self.has_get,
        }
    }
}

impl evmap::ShallowCopy for Class {
    #[inline]
    unsafe fn shallow_copy(&mut self) -> Self {
        self.clone()
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for Class {}

impl Php {
    pub fn new() -> Php {
        let (classes_r, mut classes_w) = evmap::new::<String, Class>();
        let classes_writer = Arc::new(Mutex::new(classes_w));
        let classes_reader_factory = classes_r.factory();
        classes_writer.lock().unwrap().refresh();

        Php {
            classes_writer,
            classes_reader_factory,
        }
    }

    pub fn load_class<'a>(&mut self, class_full_name: &str) -> Option<()> {
        let mut classes_r = self.classes_reader_factory.handle();

        let class = classes_r.get_and(class_full_name, |v| ());
        if class.is_none() {
            if let Some(class_path) = resolve_namespace::resolve_namespace(class_full_name) {
                self.add_from_php(&class_path);
                if let Some(_parent) = classes_r.get_and(class_full_name, |r| ()) {
                    return Some(());
                }
            }
            return None;
        }
        return Some(());
    }
}
