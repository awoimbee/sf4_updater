use std::fs;

/// finds files inside `root` w/ names ending in `ends_with`
/// And call `callback` on them.
pub fn f_find(root: &str, ends_with: &str, mut callback: impl FnMut(&str) + Clone) {
    let meta = match fs::metadata(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {} ('{}')", e, root);
            return;
        }
    };
    if meta.is_file() {
        if root.ends_with(ends_with) {
            callback(root);
        }
        return;
    }
    let entries = match fs::read_dir(&root) {
        Ok(e) => e,
        Err(_e) => {
            eprintln!("Could not read directory {}", root);
            return;
        }
    };
    for entry in entries {
        let entry = entry.unwrap().path().to_str().unwrap().to_owned();
        f_find(&entry, ends_with, callback.clone());
    }
}
