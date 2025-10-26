pub mod errors;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod std_lib;
pub mod tests;

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

    #[arg(short, long, default_value_t = true)]
    autofix: bool,
}

fn debug_print(debug: &bool, msg: &str) {
    if *debug {
        println!("[DEBUG] {}", msg);
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let code = std::fs::read_to_string(&args.filename).expect("Could not read file");
    let debug = args.debug;
    let autofix = args.autofix;

    debug_print(&debug, "Starting interpretation process...");
    debug_print(&debug, "Reading input code...");

    let code = if autofix {
        debug_print(&debug, "Autofix enabled, fixing code...");
        let fixed_code = lexer::autofix(&code);
        if fixed_code != code {
            debug_print(&debug, "Code was modified by autofix.");
            // write the fixed code back to the file
            std::fs::write(&args.filename, &fixed_code)
                .expect("Could not write fixed code back to file");
        } else {
            debug_print(&debug, "No changes made by autofix.");
        }
        fixed_code
    } else {
        code
    };

    debug_print(&debug, "Input code:");
    debug_print(&debug, code.as_str());

    // Tokenize
    let tokens = tokenize(code.to_string());
    debug_print(&debug, "\nTokens:");
    let tokens_as_tokentype: Vec<_> = tokens.iter().map(|token| token.token.to_owned()).collect();

    // debug
    if debug {
        println!("Tokens:");
        for token in &tokens {
            println!("-> {:?}", token);
        }
    }

    let parse_result = parse(&tokens).unwrap();
    debug_print(&debug, "\nParsed successfully.");
    debug_print(&debug, format!("AST: {:#?}", parse_result).as_str());

    // debug_print!("\nParsed AST:");
    debug_print(&debug, "\nInterpreting:");
    debug_print(
        &debug,
        "-------------------------------------------------------------",
    );
    interpreter::interpret(&parse_result);
    debug_print(
        &debug,
        "-------------------------------------------------------------",
    );
}
