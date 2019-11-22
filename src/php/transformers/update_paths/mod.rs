mod dealias_path;
mod update_view_path;

use super::super::Php;
use super::FileTransformer;
use crate::f_find::f_find;
use crate::G;
use dealias_path::dealias_path;
use update_view_path::update_view_path;
use regex::Regex;
use std::sync::Mutex;
use std::process::Command;
use futures::future::join_all;
use futures::executor::block_on;


lazy_static!{
    /// SHELL_COMMANDS[0] == ["git", "-C", "../projet", "mv", "<src>", "<dst>"]
    static ref SHELL_COMMANDS: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());
}

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
///     ./src/path/filename.type

impl Php {
    pub fn update_paths(self) {
        println!("update_twig_paths");

        f_find(&G.work_dir, r".*\.(php|twig|yml|yaml|xml)", |s| {
            foreach_path(s)
        });

        // fuk fuk fuk
        f_find(&format!("{}/app", &G.project_root), r".*\.(php|twig|yml|yaml|xml)", |s| {
            foreach_path(s)
        });

        consume_shell_commands();

    }
}

fn consume_shell_commands() {
    println!("Executing shell commands...");
    let mut g_cmds_lock = SHELL_COMMANDS.lock().unwrap();
    let mut cmds_stack = Vec::with_capacity(g_cmds_lock.len());

    let cmd_async = |cmd: Vec<String>|  async move {
        let out = Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .expect("failed to execute process");
        return (cmd.join(" "), out);
    };
    while let Some(cmd) = g_cmds_lock.pop() {
        cmds_stack.push(cmd_async(cmd));
    }
    for (cmd, cmd_res) in block_on(join_all(cmds_stack)){
        let mut out = cmd + "\n";
        if cmd_res.stdout.len() > 0 {
            let stdout = std::str::from_utf8(&cmd_res.stdout).unwrap();
            out.push_str(&format!("\tstdout: {}\n", stdout));
        }
        if cmd_res.stderr.len() > 0 {
            let stderr = std::str::from_utf8(&cmd_res.stderr).unwrap();
            out.push_str(&format!("\tstderr: {}\n", stderr));
        }
        print!("{}", out);
    }
}

fn foreach_path(file: &str) {
    lazy_static! {
        static ref RE_PATH: Regex = build_path_regex();
    }

    let mut ft = FileTransformer::new(&file);

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

        if path.contains("Resources/views") {
            match update_view_path(&path) {
                Ok(path) => ft.reader_replace(start, end, &path),
                Err(e) => {
                    println!("{} ({})", e, &path_cap[0]);
                    ft.reader_skip(end);
                    continue;
                }
            }
            // ft.reader_replace(start, end, &path);
            // println!("View: '{:70}' -> '{}'", &path_cap[0], path);
            // println!("new_path: {}", );
            // ft.reader_replace(start, end, &update_view_path(&path));
        } else {
            println!("?: '{:70}' -> '{}'", &path_cap[0], path);
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
            r#"(?P<colon>{root_nspace}(?P<colon_bundle>{bundles})Bundle:(?P<colon_path>[^: ]*):(?P<colon_file>[^'\s ]*))"#,
            r#"|(?P<shortBundle>@{root_nspace}(?P<shortBundle_bundle>{bundles})Bundle/(?P<shortBundle_path>[^'\s]*))"#,
            r#"|(?P<short>@{root_nspace}(?P<short_bundle>{bundles})/(?P<short_path>[^'\s]*))"#,
            // r#"|(?P<std>['"][^ \n\t\v\r:;`]*?(?P<std_bundle>{bundles})Bundle[^ \n\t\v\r:;`]*?['"])"#
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
