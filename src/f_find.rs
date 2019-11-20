use std::fs;

/// finds files inside `root` w/ names ending in `ends_with`
/// And call `callback` on them.
pub fn f_find(root: &str, ends_with: &str, mut callback: impl FnMut(&str)) {
    let mut dir_stack = vec!(root.to_owned());
    while dir_stack.len() != 0 {
        let dir = dir_stack.pop().unwrap();
        let dir_meta = match fs::metadata(&dir) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Could not get info for file {} ({})", dir, e);
                continue;
            }
        };
        if dir_meta.is_file() && dir.ends_with(ends_with) {
            callback(&dir);
        } else if dir_meta.is_dir() {
            let sub = match fs::read_dir(&dir) {
                Ok(s) => s.map(|d| d.unwrap().path().to_str().unwrap().to_owned()),
                Err(e) => {
                    eprintln!("Could not read directory {} ({})", dir, e);
                    continue;
                }
            };
            dir_stack.extend(sub);
        }
    }
}
