use crate::G;
use std::collections::HashMap;
use std::fs;

/// Returns a strign containing the full path or the path relative
/// to the curent executable
pub fn dealias_path(path: &regex::Captures<'_>, file_loc: &str) -> Result<String, &'static str> {
    match () {
        _ if path.name("colon").is_some() => dealias_colon_path(path),
        _ if path.name("shortBundle").is_some() => dealias_shortbundle_path(path),
        _ if path.name("short").is_some() => dealias_short_path(path),
        _ if path.name("std").is_some() => dealias_std_path(path, file_loc),
        _ => Err("Unknown path format"),
    }
}

/// Also accepts `path/to/controller:action`
fn normalize_path_str(path: &mut String) {
    // Welcome back to C
    unsafe {
        let mut bytes = path.as_bytes_mut();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] as char == '\\' {
                bytes[i] = '/' as u8;
                continue;
            }
            if bytes[i] as char == '/' {
                if i == bytes.len() - 1 {
                    return;
                }
                if bytes[i + 1] as char == '/' || bytes[i + 1] as char == '\\' {
                    path.remove(i);
                    bytes = path.as_bytes_mut();
                    continue;
                }
            }
            i += 1;
        }
    }
}

/// Returns the path to a bundle of name `bundle_name`
fn bundle_path(bundle_name: &str) -> Result<&str, &'static str> {
    lazy_static! {
        static ref BUNDLE_FDS: HashMap<String, String> = {
            let mut b_map = HashMap::new();
            for dir in fs::read_dir(&G.project_srcs).unwrap() {
                let dir = dir.unwrap();
                let d_name = dir.file_name();
                let d_name = d_name.to_string_lossy();
                if !d_name.ends_with("Bundle") {
                    continue;
                }
                let b_name = &d_name[..d_name.rfind("Bundle").unwrap()];
                let d_path = dir.path().to_str().unwrap().to_owned();
                b_map.insert(b_name.to_owned(), d_path);
            }
            b_map
        };
    }
    match BUNDLE_FDS.get(bundle_name) {
        Some(p) => Ok(p),
        None => Err("Could not get path of bundle: Unknown bundle"),
    }
}

/// Returns path from colon alias
fn dealias_colon_path(colon_alias: &regex::Captures<'_>) -> Result<String, &'static str> {
    let bundle = colon_alias.name("colon_bundle").unwrap().as_str();
    let midle = colon_alias.name("colon_path").unwrap().as_str();
    let last = colon_alias.name("colon_file").unwrap().as_str();

    let bundle_path = bundle_path(bundle)?;
    let mut path = match last.contains('.') {
        true => format!("{}/Resources/views/{}/{}", bundle_path, midle, last),
        false => format!(
            "{}/Controller/{}Controller.php:{}",
            bundle_path, midle, last
        ),
    };
    normalize_path_str(&mut path);
    Ok(path)
}

/// Returns path from short bundle alias
fn dealias_shortbundle_path(sbundle: &regex::Captures<'_>) -> Result<String, &'static str> {
    let bundle = sbundle.name("shortBundle_bundle").unwrap().as_str();
    let path = sbundle.name("shortBundle_path").unwrap().as_str();

    let bundle_path = bundle_path(bundle)?;
    let mut path = format!("{}/{}", bundle_path, path);
    normalize_path_str(&mut path);
    Ok(path)
}

/// Returns path from bundle alias
fn dealias_short_path(bundle_re: &regex::Captures<'_>) -> Result<String, &'static str> {
    let bundle = bundle_re.name("short_bundle").unwrap().as_str();
    let path = bundle_re.name("short_path").unwrap().as_str();

    let bundle_path = bundle_path(bundle)?;
    let mut path = format!("{}/Resources/views/{}", bundle_path, path);
    normalize_path_str(&mut path);
    Ok(path)
}

fn dealias_std_path(path_cap: &regex::Captures<'_>, loc: &str) -> Result<String, &'static str> {
    // if path_cap[0].starts_with("./src") {}

    Ok(path_cap[0].to_owned())
    // "Megapute, Octopute, Hydropute, Triplepute, Aquapute"
}
