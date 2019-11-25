mod dealias_path;
mod update_view_path;
mod file_mover;

use super::super::Php;
use super::FileTransformer;
use crate::f_find::f_find;
use crate::G;
use file_mover::FileMover;
use dealias_path::dealias_path;
use update_view_path::update_view_path;
use regex::Regex;

/// Colon path:
///     RootMyBundle:path/inside/Resources/views:filename.type
///     RootMyBundle:path/inside/Contoller/file:action
/// Short bundle path:
///     @RootMy/path/inside/Resources/views/filename.type
/// Bundle path:
///     @RootMyBundle/path/filename.type
/// std path:
///     %kernel.project_dir%/src/path/filename.type
///     ../../../filename.type
///     ./(IN_PROOT-IN_CUR_DIR)/path/filename.type
///     src/path/filename.type
///     src/path/filename
/// meeroshoot / meeroshowcase -> src/*Bundle/Resources/
/// /bundles/meeroshoot <- s3 assets ?

impl Php {
    pub fn update_paths(self) {
        println!("update_paths");
        let mut fm: FileMover = FileMover::new();

        f_find(&G.work_dir, r".*", |s| {
            foreach_path(s, &mut fm)
        });
        // paths also need to be updated in ./app/
        f_find(&format!("{}/app", &G.project_root), r".*", |s| {
            foreach_path(s, &mut fm)
        });
        fm.git_mv();
    }
}



fn foreach_path(file: &str, fm: &mut FileMover) {
    lazy_static! {
        static ref RE_PATH: Regex = build_path_regex();
    }

    let mut ft = match FileTransformer::new(&file) {
        Some(ft) => ft,
        None => return,
    };

    while let Some(path_cap) = RE_PATH.captures(ft.reader()) {
        let start = path_cap.get(0).unwrap().start();
        let end = path_cap.get(0).unwrap().end();
        let path = match dealias_path(&path_cap) {
            Ok(p) => p,
            Err(e) => {
                println!("\tCould not transform path ({}): {}", e, &path_cap[0]);
                ft.reader_skip(end);
                continue;
            }
        };
        // path to controller -> namespace (`controller: App\Controller\BlogController::list`)
        // path to twig -> relative file path from `templates/` (app/Resources/views at first)
        // path to mjml -> relative path to project_dir ? (gilpfile.js: gulp.task('mjml', function() {)

        if path.contains("/Controller/") {

        }


        if path.contains("Resources/views") {
            match update_view_path(&path, fm) {
                Ok(path) => ft.reader_replace(start, end, &path),
                Err(e) => {
                    println!("{} ({})", e, &path_cap[0]);
                    ft.reader_skip(end);
                    continue;
                }
            }
        } else {
            println!("?: '{:81}' -> '{}'", &path_cap[0], path);
            ft.reader_skip(end);
        }
    }
    ft.write_file(file);
}

fn build_path_regex() -> Regex {
    let bundles_reg_names = G.bundles.iter().map(|(n, _)| format!("{}|", n));
    let mut bundles_reg_str = bundles_reg_names.collect::<String>();
    bundles_reg_str.pop();
    let reg_str = format!(
        concat!(
            r#"(?P<colon>{root_nspace}(?P<colon_bundle>{bundles})Bundle:(?P<colon_path>[^: ]*):(?P<colon_file>[^()"'\s ]*))"#,
            r#"|(?P<shortBundle>@{root_nspace}(?P<shortBundle_bundle>{bundles})Bundle/(?P<shortBundle_path>[^()"'\s]*))"#,
            r#"|(?P<short>@{root_nspace}(?P<short_bundle>{bundles})/(?P<short_path>[^()"'\s]*))"#,
            // r#"|(?P<std>['"][^ \n\t\v\r:;`]*?(?P<std_bundle>{bundles})Bundle[^ \n\t\v\r:;`]*?['"])"#
            r#"|(?P<std>['"](?P<root>%kernel\.project_dir%/)?(?P<path>(?:(?:\.|\.\.|[A-Za-z\-_0-9]*)/)+)(?P<file_name>[A-Za-z\-_0-9.]*)(?::(?P<action>[A-Za-z\-_0-9]*))?['"])"#,
        ),
        bundles = bundles_reg_str,
        root_nspace = G.root_namespace
    );
    match Regex::new(&reg_str) {
        Ok(r) => r,
        Err(e) => panic!(
            "Could not build regex from bundle name ({}): {}",
            reg_str, e
        ),
    }
}
