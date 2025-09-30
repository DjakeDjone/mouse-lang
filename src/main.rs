pub mod interpreter;
pub mod lexer;
pub mod parser;

use lexer::tokenize;
use parser::parse;

fn main() {
    // Test the parser with some sample MouseLang code
    let code = r#"
        fn add(a, b) {
            return a + b;
        }

        let result = add(5, 3);
        print(result); // Should print 8

        fn factorial(n) {
            if n <= 1 {
                return 1;
            }
            return n * factorial(n - 1);
        }

        let fact3 = factorial(3);
        print("Factorial result:");
        print(fact3);

        print("Hello, MouseLang!");
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
