pub mod interpreter;
pub mod lexer;
pub mod parser;

use lexer::tokenize;
use parser::parse;

fn main() {
    // Test the parser with some sample MouseLang code
    let code = r#"
        let x = 5;
        let y = 10;
        let z = x + y;

        fn add(a, b) {
            return a + b;
        }

        fn multiply(a, b) {
            return a * b;
        }
        // Call the function
        let result = add(x, y);
        print(result);
        print("------------");
        let product = multiply(x, y);
        print(product);

        fn factorial(n) {
            if n == 0 {
                return 1;
            } else {
                return n * factorial(n - 1);
            }
        }

        let fact5 = factorial(5);
        print(fact5);
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
