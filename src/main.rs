use std::{
    process::{
        self,
        Command,
        Output,
        ExitStatus,
    },
    path::Path,
    io::{
        self,
        Write,
        BufRead,
        Error,
        ErrorKind,
    },
    panic::{
        self,
        AssertUnwindSafe
    },
};

struct Ctx {
    Directory: String,
    ExitCmd: String,
    ChangeDirectoryCmd: String,
    CharacterLimit: usize,
}

fn main() -> io::Result<()> {
    let mut ctx = Ctx {
        Directory: ".".to_owned(),
        ExitCmd: "exit".to_owned(),
        ChangeDirectoryCmd: "cd".to_owned(),
        CharacterLimit: 1000,
    };
    loop {
        let mut ctx_wrapper = AssertUnwindSafe(&mut ctx);
        let result = panic::catch_unwind(move || {
            let cmd_result = execute_cmd(&mut ctx_wrapper);
            
        });

    }
}

fn execute_cmd(ctx: &mut AssertUnwindSafe<&mut Ctx>) -> io::Result<Output> {
    let mut input = String::new();
    let mut handle_in = io::stdin().lock();
    let mut handle_out = io::stdout().lock();
    let mut handle_err = io::stderr().lock();
    handle_err.write_all(b"$ ")?;
    handle_in.read_line(&mut input)?;
    let parsed_input: Vec<&str> = input.trim().split_whitespace().map(|s| { s.trim() }).collect();
    let (cmd_option, args_option): (Option<&str>, Option<&[&str]>) = match &parsed_input[..] {
        [cmd, args @ ..] => { 
            if args.len() > 0 {
                (Some(cmd), Some(args))
            } else {
                (Some(cmd), None)
            }
         },
        [] => { ( None, None) }
    };
    #[cfg(debug_assertions)]
    println!("parsed_input {:?}", parsed_input);
    // TODO implement print errors, cd, exit (with exit code if non 0), exit when EOF on stdout, 1000 characters limit length and error, 
    if cmd_option == None {
        // do no command
        Ok(
            Output {
                status: ExitStatus::default(),
                stdout: Vec::<u8>::from([]),
                stderr: Vec::<u8>::from([]),
            }
        )
    } else {
        let cmd = cmd_option.expect("Case None command must be considered.").to_lowercase();
        let result: io::Result<Output> = if cmd == ctx.ExitCmd {
            handle_out.flush();
            handle_err.flush();
            match args_option {
                None => {
                    process::exit(0) // exit code 0_i32
                },
                Some(args) => {
                    match args[0].parse::<i32>() {
                        Ok(exit_code) => {
                            process::exit(exit_code)
                        },
                        Err(e) => {
                            process::exit(42)
                        }
                    }
                },
            }
        } else if cmd == ctx.ChangeDirectoryCmd {
            match args_option {
                None => {
                    // do no change directory. TODO: document on README.md
                    Ok(())
                },
                Some(args) => {
                    if Path::new(args[0]).exists() {
                        ctx.Directory = args[0].to_owned();
                        Ok(())
                    } else { 
                        Err(ErrorKind::NotFound)
                    }
                },
            };
            Ok(
                Output {
                    status: ExitStatus::default(),
                    stdout: Vec::<u8>::from([]),
                    stderr: Vec::<u8>::from([]),
                }
            )
        } else {
            let mut command = Command::new(cmd);
            match args_option {
                None => {
                    command.current_dir(&ctx.Directory).output()
                },
                Some(args) => {
                    command.current_dir(&ctx.Directory).args(args).output()
                },
            }
        };
        match result {
            Ok(ref output) => {
                if output.status.success() {
                    handle_out.write_all(&output.stdout);
                    handle_out.flush();
                } else {
                    let message = format!("error: command exited with {}\n", output.status);
                    handle_err.write_all(&output.stderr);
                    handle_err.write_all(&message.into_bytes());
                }
            },
            Err(ref e) => {
                let error = format!("{}\n",e);
                handle_err.write_all(&error.into_bytes());
                
            }
        }
        result
    }
}

fn exit(exit_code: i32) {
    process::exit(exit_code)
}