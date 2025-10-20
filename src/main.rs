pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod std;

use lexer::tokenize;
use parser::parse;

#[tokio::main]
async fn main() {
    // Socket server example
    let code = r#"
        # Socket Server Example for MouseLang

        fn onConnect(clientId) {
            print("Client connected:");
            print(clientId);
        }

        fn onMessage(clientId, message) {
            print("Received message from:");
            print(clientId);
            print("Message:");
            print(message);
            return "Echo: " + message;
        }

        fn onDisconnect(clientId) {
            print("Client disconnected:");
            print(clientId);
        }

        print("Starting socket server...");
        std.socketServer("127.0.0.1", 8080, onConnect, onMessage, onDisconnect);
        print("Server is running on 127.0.0.1:8080");
        print("Connect with: nc 127.0.0.1 8080 or telnet 127.0.0.1 8080");

        # Keep the main thread alive for a bit to allow connections
        let i = 0;
        while i < 100000 {
            i = i + 1;
            std.sleep(1000);
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
