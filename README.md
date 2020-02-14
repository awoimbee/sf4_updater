# sf4_updater
Helper for migrating from symfony 3 to symfony 4. Scans and removes deprecations (symfony 3.4 to 4 upgrade)

I think the usage is self explanatory:
```
$ cargo build --release
$ ./target/release/sf4_updater --help
myapp 0.2
Arthur W. <arthur.woimbee@gmail.com>
Helps you to update your sf3 project to sf4 & higher

USAGE:
    sf4_updater [FLAGS] [OPTIONS]

FLAGS:
    -A, --dealias_getrepo    Transformer: dialias `getRopository()` statements
    -B, --rm_get             Transformer: remove `container->get()` statements
    -h, --help               Prints help information
    -V, --version            Prints version information

OPTIONS:
    -c, --conf_file <CONF_FILE>                     Conf. file to use
    -y, --controllers_yml <CONTROLLERS_CONF_YML>    Path to file where controllers conf will be written
    -C, --dealias_paths <DEALIAS_PATHS>             Transformer: 1: moe templates 2: update contrls paths
    -r, --project_root <PROJECT_FD>                 Path to your symfony project
    -w, --work_dir <WORK_DIR>                       Dir under which modifications will be done

```

This was written for a company, on company time (so it's not the prettiest).
