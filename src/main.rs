#[macro_use]
extern crate lazy_static;
extern crate getopts;

mod activity;
mod error;
mod interpreter;
mod logger;
mod registry;
mod script;

use registry::Registry;
use std::process::exit;

static DEFAULT_LOG_FILE: &str = "activity.log";

fn print_usage(program: &str, opts: getopts::Options, registry: &Registry) {
    let brief = format!("Usage: {} [options] <script>", program);
    print!("{}", opts.usage(&brief));
    println!("\nRegistered functions:");
    for e in registry.funcs() {
        println!(
            "    {} {}",
            e.name,
            e.args
                .iter()
                .map(|a| if let Some(stripped) = a.strip_prefix('*') {
                    format!("[{}, ...]", stripped)
                } else {
                    format!("<{}>", a)
                })
                .collect::<Vec<String>>()
                .join(" ")
        );
    }
}

struct Opts {
    script: String,
    script_is_source: bool,
    log_file: String,
}

fn parse_opts(registry: &Registry) -> Option<Result<Opts, String>> {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();

    opts.parsing_style(getopts::ParsingStyle::StopAtFirstFree)
        .optopt(
            "l",
            "",
            &format!("set log file (default: {})", DEFAULT_LOG_FILE),
            "<file>",
        )
        .optflag(
            "c",
            "",
            "execute script string (don't read from a file), \
            arguments are implicitly quoted",
        )
        .optflag("h", "", "show this message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            return Some(Err(f.to_string()));
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts, registry);
        return None;
    }

    let script_is_source = matches.opt_present("c");

    let log_file = matches
        .opt_str("l")
        .map_or_else(|| DEFAULT_LOG_FILE.to_string(), |v| v);

    let script = if matches.free.len() == 1 {
        matches.free[0].clone()
    } else if script_is_source && !matches.free.is_empty() {
        matches
            .free
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                if i == 0 || arg.contains('"') {
                    arg.into()
                } else {
                    format!("\"{}\"", arg)
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    } else {
        print_usage(&program, opts, registry);
        return None;
    };

    Some(Ok(Opts {
        script,
        script_is_source,
        log_file,
    }))
}

fn exec<R: std::io::Read>(src: R, registry: &registry::Registry) -> Result<(), error::Error> {
    let ast = match script::parse(std::io::BufReader::new(src), &registry) {
        Ok(ast) => ast,
        Err(err) => {
            return Err(err);
        }
    };

    interpreter::interp(&ast)?;

    Ok(())
}

fn main() {
    let mut registry = Registry::default();
    activity::register(&mut registry);

    let opts = match parse_opts(&registry) {
        Some(thing) => match thing {
            Ok(opts) => opts,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        },
        None => {
            return;
        }
    };

    logger::init(&opts.log_file);

    use std::fs::File;

    let res = if opts.script_is_source {
        exec(opts.script.as_bytes(), &registry)
    } else {
        match File::open(&opts.script) {
            Ok(f) => exec(f, &registry),
            Err(e) => {
                eprintln!("{}", e);
                exit(1);
            }
        }
    };

    match res {
        Ok(_) => (),
        Err(err) => {
            err.display(&opts.script, opts.script_is_source);
            exit(1);
        }
    }
}
