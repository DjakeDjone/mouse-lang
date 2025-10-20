pub mod interpreter;
pub mod lexer;
pub mod parser;

use lexer::tokenize;
use parser::parse;

fn main() {
    let code = r#"
        fn add(a, b) {
            return a + b;
        }

        let result = add(5, 3);
        print(result); // Should print 8

        function factorial(n) {
            if n <= 1 {
                return 1;
            }
            return n * factorial(n - 1);
        }

        let fact3 = factorial(3);
        print("Factorial result:");
        print(fact3);

        print("Hello, MouseLang!");

        fn print_12_times(message, times) {
            print(message);
            if times > 0 {
                print_12_times(message, times - 1);
            }
        }

        print_12_times("Hello, MouseLang!", 12);

        # loops
        # print($i).for(0, 12)
        # nested loops
        # print($i + ":" + $i1).for(0, 12).for(0, 12)

        let i = 0;
        while (i < 5) {
            print(i);
            i = i + 1;
        }

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
    println!("-------------------------------------------------------------");
    interpreter::interpret(&parse_result);
    println!("-------------------------------------------------------------");
}
