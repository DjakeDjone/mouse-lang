pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod std_lib;

use clap::Parser;
use lexer::tokenize;
use parser::parse;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the file to process
    #[arg(short, long)]
    filename: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn debug_print(debug: &bool, msg: &str) {
    if *debug {
        println!("[DEBUG] {}", msg);
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let code = std::fs::read_to_string(args.filename).expect("Could not read file");
    let debug = args.debug;

     debug_print(&debug, "Starting interpretation process...");
     debug_print(&debug, "Reading input code...");

    debug_print(&debug, "Input code:");
    debug_print(&debug,  code.as_str());

    // Tokenize
    let tokens = tokenize(code.to_string());
    debug_print(&debug, "\nTokens:");

    let parse_result = parse(&tokens)
        .map_err(|e| {
            eprintln!("\nParse error:");
            for err in &e {
                eprintln!("  {:?}", err);
            }
            e
        })
        .unwrap();
    debug_print(&debug, "\nParsed successfully.");
    debug_print(&debug, format!("AST: {:#?}", parse_result).as_str());

    // debug_print!("\nParsed AST:");
    debug_print(&debug, "\nInterpreting:");
    debug_print(&debug, "-------------------------------------------------------------");
    interpreter::interpret(&parse_result);
    debug_print(&debug, "-------------------------------------------------------------");
}
