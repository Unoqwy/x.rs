use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, fs, io};

use config::*;
use find::*;

mod config;
mod find;
mod parser;

macro_rules! fail {
    ($($tt:tt)*) => {
        eprintln!($($tt)*);
        std::process::exit(1);
    };
}

macro_rules! usage {
    ($usage:expr, $desc:expr) => {
        fail!(concat!("Usage: ", $usage, "\nDescription: ", $desc));
    };
}

fn main() -> io::Result<()> {
    let initial_cwd = env::current_dir()?;
    let (root_dir, config_file) = find_root(None)?;
    env::set_current_dir(&root_dir)?;
    let contents = fs::read_to_string(&config_file)?;
    let root_config: RootConfiguration = match config_file.extension().and_then(|s| s.to_str()) {
        Some("yml" | "yaml") => serde_yaml::from_str(&contents).unwrap(),
        Some("kdl") => parser::parse_kdl_configuration(&contents),
        _ => unreachable!("CONFIG_FILENAMES has been updated without updating this handler"),
    };

    if env::args().nth(1).map(|s| s.eq("-")).unwrap_or(false) {
        builtin_mode(&root_config, &config_file)?;
    } else {
        match root_config.mode {
            RootMode::Standalone => standalone_mode(&root_config, &initial_cwd)?,
            RootMode::Passthrough { .. } => passthrough_mode(&root_config, &initial_cwd)?,
        }
    }

    Ok(())
}

fn builtin_mode(root_config: &RootConfiguration, config_file: &PathBuf) -> io::Result<()> {
    let mut args = env::args().skip(2);
    match args.next().as_deref() {
        Some("which") => {
            if let Some(script) = args.next() {
                if root_config.scripts.contains_key(&script) {
                    println!("{}: project scoped script", script);
                } else if let Some(binary) = find_binary(&root_config.hoist, &script)? {
                    println!(
                        "{}: hoisted from {}",
                        script,
                        fs::canonicalize(binary)?.to_str().unwrap()
                    );
                } else {
                    fail!("{}: not found", script);
                }
            } else {
                usage!(
                    "x - which <script>",
                    "Get absolute path of hoisted binary or check if script is defined"
                );
            }
        }
        Some("root") => {
            println!(
                "Project root found at {}",
                fs::canonicalize(config_file)?.to_str().unwrap()
            );
        }
        Some("print-config") => {
            println!("{:#?}", root_config);
        }
        Some(s) => {
            fail!("Unknown builtin command {}", s);
        }
        None => {
            usage!("x - <command> [arguments...]", "x.rs built-in commands");
        }
    }
    Ok(())
}

fn standalone_mode(root_config: &RootConfiguration, initial_cwd: &PathBuf) -> io::Result<()> {
    let mut args = env::args().skip(1);
    let script: &str = &args.next().map(|s| s.to_lowercase()).unwrap_or_else(|| {
        fail!("Usage: x <script> [arguments...]");
    });

    if let Some(script_decl) = root_config.scripts.get(script) {
        let remaining_args: Vec<String> = args.collect();
        for script_cmd in script_decl.0.iter() {
            let mut args = remaining_args.clone();

            let root_cwd = env::current_dir()?;
            env::set_current_dir(initial_cwd)?;
            let cmd = script_command_to_shell(&script_cmd, &mut args);
            env::set_current_dir(root_cwd)?;

            let mut command = spawn_shell(&cmd);
            command.args(args);
            if let Some(rel) = &script_cmd.cwd {
                command.current_dir(fs::canonicalize(rel)?);
            } else {
                command.current_dir(initial_cwd);
            }
            command.output().unwrap();
        }
    } else {
        if let Some(binary) = find_binary(&root_config.hoist, script)? {
            let mut command = spawn_shell(binary.canonicalize()?.to_str().unwrap());
            command.output().unwrap();
        } else {
            fail!("Unknown script or hoisted binary '{}'", script);
        }
    }
    Ok(())
}

fn script_command_to_shell(cmd: &ScriptCmd, args: &mut Vec<String>) -> String {
    let ScriptCmd {
        cmd, process_args, ..
    } = cmd;
    *args = args
        .iter()
        .enumerate()
        .map(|(i, arg)| match process_args.get(&((i + 1) as u16)) {
            Some(ProcessArgument::Transform { transform }) => match transform as &str {
                "realpath" => fs::canonicalize(arg).unwrap().to_str().unwrap().to_owned(),
                _ => unimplemented!("Unknown transform {}", transform),
            },
            None => arg.clone(),
        })
        .collect();
    cmd.clone()
}

fn passthrough_mode(root_config: &RootConfiguration, initial_cwd: &PathBuf) -> io::Result<()> {
    if let RootMode::Passthrough {
        cmd,
        cwd,
        prepend_args,
        append_args,
    } = &root_config.mode
    {
        let args: Vec<String> = env::args().skip(1).collect();
        let mut command = Command::new(cmd);
        command
            .args(prepend_args)
            .args(args)
            .args(append_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        if let Some(rel) = &cwd {
            command.current_dir(fs::canonicalize(rel)?);
        } else {
            command.current_dir(initial_cwd);
        }

        command.output().unwrap();
    }
    Ok(())
}

fn spawn_shell(cmd: &str) -> Command {
    let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_owned());
    let mut command = Command::new(&shell);
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("-c")
        .arg(cmd)
        .arg(env::args().nth(0).unwrap());
    command
}
