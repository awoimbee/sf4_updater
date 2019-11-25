use regex::Regex;
use std::fs;

// If performance becomes an issue: create FileFinder object & construct index

/// finds files inside `root` w/ names that matches
/// And call `callback` on them.
pub fn f_find(root: &str, regex_match: &'static str, mut callback: impl FnMut(&str)) {
    let reg = Regex::new(regex_match).unwrap();
    let mut dir_stack: Vec<String> = Vec::new();
    // let mut file_stack: Vec<String> = Vec::new();

    match fs::metadata(root) {
        Ok(m) if m.is_dir() => dir_stack.push(root.to_owned()),
        Ok(m) if m.is_file() && reg.is_match(root) => callback(root),
        Err(e) => {
            eprintln!("Could not get info for file {} ({})", root, e);
            return;
        }
        _ => return
    };

    while let Some(dir) = dir_stack.pop() {
        let sub = match fs::read_dir(&dir) {
            Ok(s) => s.map(|d| d.unwrap().path().to_str().unwrap().to_owned()),
            Err(e) => {
                eprintln!("Could not read directory {} ({})", dir, e);
                continue;
            }
        };
        for s in sub {
            let s_meta = match fs::metadata(&s) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Could not get info for file {} ({})", dir, e);
                    continue;
                }
            };
            match () {
                _ if s_meta.is_dir() => dir_stack.push(s),
                _ if s_meta.is_file() && reg.is_match(&s) => callback(&s),
                _ => ()
            };
        }
    }
}
