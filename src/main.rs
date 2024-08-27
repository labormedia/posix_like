use std::{
    env,
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
mod selector;
use selector::Selector;

struct Ctx {
    current_directory: String,
    exit_cmd: String,
    change_directory_cmd: String,
    character_limit: usize,
    argument_limit: usize,
}

fn main() -> io::Result<()> {
    let mut ctx = Ctx {
        current_directory: ".".to_owned(),
        exit_cmd: "exit".to_owned(),
        change_directory_cmd: "cd".to_owned(),
        character_limit: 1000,
        argument_limit: 100,
    };
    let path = Path::new(&ctx.current_directory);
    if path.exists() {
        env::set_current_dir(path)?
    } else {
        panic!("Invalid setup.");
    }
    loop {
        let mut ctx_wrapper = AssertUnwindSafe(&mut ctx);
        let result = panic::catch_unwind(move || {
            let cmd_result = execute_cmd(&mut ctx_wrapper);
            match cmd_result {
                Ok(payload) => {
                    #[cfg(debug_assertions)]
                    println!("Commmand execution result {:?}", payload);
                },
                Err(e) => {
                    #[cfg(debug_assertions)]
                    println!("Command execution error {:?}", e);
                },
            }
        });
        match result {
            Ok(payload) => {
                #[cfg(debug_assertions)]
                println!("Unwinded result {:?}", payload);
            },
            Err(e) => {
                #[cfg(debug_assertions)]
                println!("Unwinded error {:?}", e);
            },
        }
    }
}

fn execute_cmd(ctx: &mut AssertUnwindSafe<&mut Ctx>) -> io::Result<Output> {
    let mut input = String::new();
    let mut selector: Selector = Selector {
        active: false,
        chars_selected: vec!['\u{0022}', '\u{0027}'], // double quotes and apostrophe
        char_stack: Vec::new(),
        catch_error: false,
    };
    let mut handle_in = io::stdin().lock();
    let mut handle_out = io::stdout().lock();
    let mut handle_err = io::stderr().lock();
    handle_err.write_all(b"$ ")?;
    handle_in.read_line(&mut input)?;
    // selector_binding is the result of the application of the selection logic
    // based on the Selector implementation.
    let input_selection: Vec<char> = input
        .trim()
        .chars()
        .collect();
    let mut selector_binding = input_selection
        .iter()
        .enumerate()
        .map( |(i,c)| {
            let selection = (i, selector.select(&c), c, selector.clone());
            println!("{selection:?}");
            selection
        })
        .fold( Ok(Vec::<String>::new()), |result_acc: io::Result<Vec<String>>, (i, selection, c, selector)| {
            match result_acc {
                Err(e) => Err(e),
                Ok(mut acc) => {
                    if acc.len() == 0 {
                        acc.push(c.to_string());
                    } else {
                        match selection {
                            Ok(true) => {
                                if let Some(last) = acc.last_mut() {
                                    last.push(*c);
                                } else {
                                    // if selection is Ok(true) and there is no accumulated values yet 
                                    // then the Selector was initialized with an active = true value.
                                    return Err(Error::new(ErrorKind::Other.into(), "Invalid Selector initialization."));
                                }
                            },
                            Ok(false) => {
                                acc.push(c.to_string())
                            },
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    // if it is the last element and selector.active is still true
                    // (or there are still characters in the stack to be matched)
                    // then return an unmatched char selection error.
                    println!("enumerator {} with length {}", i, input_selection.len());
                    if i == input_selection.len() - 1 && selector.char_stack.len() > 0 {
                        Err(Error::new(ErrorKind::Other.into(), "unmatched quotes."))
                    } else {
                        Ok(acc)
                    }
                }
            }
        })?;
    // Removes any empty of single spaces that could have remained from the non destructive selection.
    selector_binding.retain( |s| {
            s != "" &&  s != " "
        });
    // Trims spaces from the remainding selections.
    let parsed_input: Vec<&str> = selector_binding
        .iter()
        .map( |s| {
            s.trim()
        })
        .collect();
    // Parses the command and the arguments.
    let (cmd_option, args_option): (Option<&str>, Option<&[&str]>) = match &parsed_input[..] {
        [cmd, args @ ..] => { 
            if cmd.len() > ctx.character_limit {
                // check size limits
                let error  = format!("{}", Error::new(ErrorKind::Other.into(), "character size limit exceeded"));
                handle_err.write_all(&error.into_bytes())?;
                (None, None)
            } else if args.len() > 0 {
                if args.len() > ctx.argument_limit {
                    let error  = format!("{}", Error::new(ErrorKind::Other.into(), "arguments size limit exceeded"));
                    handle_err.write_all(&error.into_bytes())?;
                    (None, None)
                } else {
                    (Some(cmd), Some(args))
                }
            } else {
                (Some(cmd), None)
            }
         },
        [] => { ( None, None) }
    };
    #[cfg(debug_assertions)]
    println!("Parsed_input {:?}", parsed_input);
    // TODO implement print errors, cd, exit (with exit code if non 0), exit when EOF on stdout, 1000 characters limit length and error, 
    if cmd_option == None {
        // do no command
        // Return a result of no execution.
        Ok(
            Output {
                status: ExitStatus::default(),
                stdout: Vec::<u8>::from([]),
                stderr: Vec::<u8>::from([]),
            }
        )
    } else {
        let cmd = cmd_option.expect("Case None command must be considered.").to_lowercase();
        let result: io::Result<Output> = 
            if cmd == ctx.exit_cmd {
                handle_out.flush()?;
                handle_err.flush()?;
                match args_option {
                    None => {
                        process::exit(0) // exit code 0_i32
                    },
                    Some(args) => {
                        match args[0].parse::<i32>() {
                            Ok(exit_code) => {
                                let exit_message = format!("exit code {}\n", exit_code);
                                handle_out.write_all(&exit_message.into_bytes())?;
                                process::exit(exit_code)
                            },
                            Err(_) => {
                                let exit_code_error_message = format!("Invalid exit code {}\n", args[0]);
                                handle_err.write_all(&exit_code_error_message.into_bytes())?;
                                process::exit(42)  // 42 will be the "Invalid exit code" exit code.
                            }
                        }
                    },
                }
            } else if cmd == ctx.change_directory_cmd {
                match args_option {
                    None => {
                        // do no change directory. TODO: document on README.md
                        Ok(
                            Output {
                                status: ExitStatus::default(),
                                stdout: Vec::<u8>::from([]),
                                stderr: Vec::<u8>::from([]),
                            }
                        )
                    },
                    Some(args) => {
                        let path = Path::new(&(args[0]));
                        if path.exists() {
                            ctx.current_directory = match path.canonicalize()?.to_str() {
                                Some(path_name) => {
                                    #[cfg(debug_assertions)]
                                    println!("Canonicalized path {}", path_name);
                                    env::set_current_dir(path_name)?;
                                    path_name.to_string()
                                },
                                None => { 
                                    let error = format!("{}\n", ErrorKind::NotFound);
                                    handle_err.write_all(&error.into_bytes())?;
                                    ctx.current_directory.clone()
                                    
                                }
                            };
                            Ok(
                                Output {
                                    status: ExitStatus::default(),
                                    stdout: Vec::<u8>::from([]),
                                    stderr: Vec::<u8>::from([]),
                                }
                            )
                        } else { 
                            Err(ErrorKind::NotFound.into())
                        }
                    },
                }
            } else {
                let mut command = Command::new(cmd);
                match args_option {
                    None => {
                        command.current_dir(&ctx.current_directory).output()
                    },
                    Some(args) => {
                        command.current_dir(&ctx.current_directory).args(args).output()
                    },
                }
            };
        // Handle the result of execution.
        match result {
            Ok(ref output) => {
                if output.status.success() {
                    handle_out.write_all(&output.stdout)?;
                    handle_out.flush()?;
                } else {
                    let message = format!("error: command exited with {}\n", output.status);
                    handle_err.write_all(&output.stderr)?;
                    handle_err.write_all(&message.into_bytes())?;
                }
            },
            Err(ref e) => {
                let error = format!("{}\n",e);
                handle_err.write_all(&error.into_bytes())?;
                
            }
        }
        // Return the result of execution to the caller.
        result
    }
}