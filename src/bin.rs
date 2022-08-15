use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// file to load chicken code from
    #[clap(short, long, value_parser)]
    file: String,

    /// input to be provided to the program
    #[clap(short, long, value_parser, default_value = "")]
    input: String,

    /// whether to provide a debugger of sorts. this lets you step through programs and view the stack
    #[clap(short, long, value_parser, default_value_t = false)]
    debug: bool,

    /// whether the Char instruction should convert to actual characters instead of HTML entities.
    /// disabled by default for compatibility
    #[clap(short, long, value_parser, default_value_t = false)]
    normal_char: bool,
}

fn main() {
    let args = Args::parse();

    let code = match std::fs::read_to_string(&args.file) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error reading file {:?}: {:?}", args.file, err);
            std::process::exit(1);
        }
    };

    match chicken::VMBuilder::from_chicken(&code)
        .input(args.input)
        .set_debug(args.debug)
        .set_normal_char(args.normal_char)
        .build()
        .run()
    {
        Ok(output) => println!("{}", output),
        Err(err) => eprintln!("{}", err),
    }
}
