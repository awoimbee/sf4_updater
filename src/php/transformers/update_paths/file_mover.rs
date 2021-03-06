use crate::G;
use futures::executor::block_on;
use futures::future::join_all;
use std::collections::HashMap;
use std::process::Command;
use std::str::from_utf8;

bitflags! {
    pub struct MoveWhat: u32 {
        const NONE = 0;
        const TEMPLATES = 0b01;
        const CONTROLLERS = 0b10;
    }
}

pub struct FileMover {
    pub which_files: MoveWhat,         // this is disgusting
    move_ops: HashMap<String, String>, // (new, old)
}

impl FileMover {
    pub fn contains_dst(&self, dst: &str) -> bool {
        self.move_ops.contains_key(dst)
    }

    pub fn insert(&mut self, src: String, dst: String) -> Option<String> {
        self.move_ops.insert(dst, src)
    }

    pub fn new() -> Self {
        FileMover {
            which_files: MoveWhat::NONE,
            move_ops: HashMap::new(),
        }
    }

    pub fn git_mv(&mut self) {
        println!("Executing shell commands...");
        let mut cmds_stack = Vec::with_capacity(self.move_ops.len());

        let cmd_async = |cmd: [String; 6]| {
            async move {
                let cmd_res = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .output()
                    .expect("failed to execute process");
                print!(
                    "{}\n{}{}",
                    cmd.join(" "),
                    match cmd_res.stdout.len() > 1 {
                        true => format!("\tstdout: {}\n", from_utf8(&cmd_res.stdout).unwrap()),
                        false => String::new(),
                    },
                    match cmd_res.stderr.len() > 1 {
                        true => format!("\tstderr: {}\n", from_utf8(&cmd_res.stderr).unwrap()),
                        false => String::new(),
                    },
                );
            }
        };
        for (old, new) in self.move_ops.drain() {
            let cmd: [String; 6] = [
                "git".to_owned(),
                "-C".to_owned(),
                G.project_root.clone(),
                "mv".to_owned(),
                old,
                new,
            ];
            cmds_stack.push(cmd_async(cmd));
        }

        block_on(join_all(cmds_stack));
        self.move_ops.drain();
    }
}
