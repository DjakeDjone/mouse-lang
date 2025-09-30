pub mod interpreter;
pub mod lexer;
pub mod parser;

use lexer::tokenize;
use parser::parse;

fn main() {
    // Test the parser with some sample MouseLang code
    let code = r#"
        fn simple_recursive(n) {
            print("Called with:");
            print(n);
            if n <= 0 {
                return 0;
            }
            return simple_recursive(n - 1);
        }
        
        let result = simple_recursive(2);
        print("Result:");
        print(result);
    "#;

    println!("Input code:");
    println!("{}", code);

    // Tokenize
    let tokens = tokenize(code.to_string());
    println!("\nTokens:");
    for token in &tokens {
        println!("  {:?}", token);
    }

    let parse_result = parse(&tokens)
        .map_err(|e| {
            eprintln!("\nParse error:");
            for err in &e {
                eprintln!("  {:?}", err);
            }
            e
        })
        .unwrap();
    println!("\nParsed successfully.");
    println!("AST: {:#?}", parse_result);

    // println!("\nParsed AST:");
    println!("\nInterpreting:");
    interpreter::interpret(&parse_result);
}
