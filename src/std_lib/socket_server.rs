// use crate::interpreter::{Interpreter, Value};
// use futures_util::{SinkExt, StreamExt};
// use tokio::net::TcpListener;
// use tokio_tungstenite::accept_async;
// use tokio_tungstenite::tungstenite::Message;

// type CallbackData = (String, Vec<String>, Vec<crate::parser::Stmt>);

// pub fn socket_server(interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, String> {
//     let (host, port, on_connect, on_message, on_disconnect) = validate_args(args)?;

//     // Clone the interpreter's environment for use in async context
//     let functions = interpreter.env.functions.clone();

//     // Spawn the WebSocket server in a new thread with tokio runtime
//     std::thread::spawn(move || {
//         let rt = tokio::runtime::Runtime::new().unwrap();
//         rt.block_on(async move {
//             let addr = format!("{}:{}", host, port);
//             println!("[WebSocketServer] Starting server on {}", addr);

//             let listener = match TcpListener::bind(&addr).await {
//                 Ok(l) => l,
//                 Err(e) => {
//                     eprintln!("[WebSocketServer] Failed to bind to {}: {}", addr, e);
//                     return;
//                 }
//             };

//             println!("[WebSocketServer] Server listening on ws://{}", addr);

//             loop {
//                 match listener.accept().await {
//                     Ok((stream, addr)) => {
//                         let client_id = format!("{}", addr);
//                         println!("[WebSocketServer] Client connecting: {}", client_id);

//                         // Clone data for this connection
//                         let on_connect = on_connect.clone();
//                         let on_message = on_message.clone();
//                         let on_disconnect = on_disconnect.clone();
//                         let functions = functions.clone();
//                         let native_functions = native_functions.clone();

//                         // Handle each WebSocket connection in a separate task
//                         tokio::spawn(async move {
//                             handle_client(
//                                 stream,
//                                 client_id,
//                                 on_connect,
//                                 on_message,
//                                 on_disconnect,
//                                 functions,
//                                 native_functions,
//                             )
//                             .await;
//                         });
//                     }
//                     Err(e) => {
//                         eprintln!("[WebSocketServer] Failed to accept connection: {}", e);
//                     }
//                 }
//             }
//         });
//     });

//     println!("[WebSocketServer] Server started successfully (non-blocking)");
//     Ok(Value::Void)
// }

// fn validate_args(
//     args: Vec<Value>,
// ) -> Result<(String, u16, CallbackData, CallbackData, CallbackData), String> {
//     if args.len() != 5 {
//         return Err(format!(
//             "socketServer expects 5 arguments (host, port, onConnect, onMessage, onDisconnect), got {}",
//             args.len()
//         ));
//     }

//     let host = match &args[0] {
//         Value::String(s) => s.clone(),
//         _ => return Err("First argument (host) must be a string".to_string()),
//     };

//     let port = match &args[1] {
//         Value::Number(n) => *n as u16,
//         _ => return Err("Second argument (port) must be a number".to_string()),
//     };

//     let on_connect = match &args[2] {
//         Value::Function(name, params, body) => (name.clone(), params.clone(), body.clone()),
//         _ => return Err("Third argument (onConnect) must be a function".to_string()),
//     };

//     let on_message = match &args[3] {
//         Value::Function(name, params, body) => (name.clone(), params.clone(), body.clone()),
//         _ => return Err("Fourth argument (onMessage) must be a function".to_string()),
//     };

//     let on_disconnect = match &args[4] {
//         Value::Function(name, params, body) => (name.clone(), params.clone(), body.clone()),
//         _ => return Err("Fifth argument (onDisconnect) must be a function".to_string()),
//     };

//     Ok((host, port, on_connect, on_message, on_disconnect))
// }

// async fn handle_client(
//     stream: tokio::net::TcpStream,
//     client_id: String,
//     on_connect: CallbackData,
//     on_message: CallbackData,
//     on_disconnect: CallbackData,
//     functions: std::collections::HashMap<String, (Vec<String>, Vec<crate::parser::Stmt>)>,
//     native_functions: std::collections::HashMap<String, crate::interpreter::NativeFn>,
// ) {
//     // Perform WebSocket handshake
//     let ws_stream = match accept_async(stream).await {
//         Ok(ws) => ws,
//         Err(e) => {
//             eprintln!(
//                 "[WebSocketServer] WebSocket handshake failed for {}: {}",
//                 client_id, e
//             );
//             return;
//         }
//     };

//     println!("[WebSocketServer] WebSocket connected: {}", client_id);

//     let (mut write, mut read) = ws_stream.split();

//     // Call onConnect callback
//     if let Err(e) = call_callback(
//         &on_connect.0,
//         &on_connect.1,
//         &on_connect.2,
//         vec![Value::String(client_id.clone())],
//         &functions,
//         &native_functions,
//     ) {
//         eprintln!("[WebSocketServer] onConnect error: {}", e);
//     }

//     // Handle incoming messages
//     while let Some(msg_result) = read.next().await {
//         match msg_result {
//             Ok(msg) => {
//                 if !handle_message(
//                     msg,
//                     &client_id,
//                     &mut write,
//                     &on_message,
//                     &functions,
//                     &native_functions,
//                 )
//                 .await
//                 {
//                     break;
//                 }
//             }
//             Err(e) => {
//                 eprintln!(
//                     "[WebSocketServer] Error receiving message from {}: {}",
//                     client_id, e
//                 );
//                 break;
//             }
//         }
//     }

//     // Connection closed
//     println!("[WebSocketServer] Client disconnected: {}", client_id);

//     // Call onDisconnect callback
//     if let Err(e) = call_callback(
//         &on_disconnect.0,
//         &on_disconnect.1,
//         &on_disconnect.2,
//         vec![Value::String(client_id.clone())],
//         &functions,
//         &native_functions,
//     ) {
//         eprintln!("[WebSocketServer] onDisconnect error: {}", e);
//     }
// }

// async fn handle_message(
//     msg: Message,
//     client_id: &str,
//     write: &mut futures_util::stream::SplitSink<
//         tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
//         Message,
//     >,
//     on_message: &CallbackData,
//     functions: &std::collections::HashMap<String, (Vec<String>, Vec<crate::parser::Stmt>)>,
//     native_functions: &std::collections::HashMap<String, crate::interpreter::NativeFn>,
// ) -> bool {
//     match msg {
//         Message::Text(text) => {
//             println!("[WebSocketServer] Received from {}: {}", client_id, text);

//             // Call onMessage callback
//             match call_callback(
//                 &on_message.0,
//                 &on_message.1,
//                 &on_message.2,
//                 vec![
//                     Value::String(client_id.to_string()),
//                     Value::String(text.clone()),
//                 ],
//                 functions,
//                 native_functions,
//             ) {
//                 Ok(response) => {
//                     if !send_response(write, response).await {
//                         return false;
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("[WebSocketServer] onMessage error: {}", e);
//                 }
//             }
//         }
//         Message::Binary(data) => {
//             println!(
//                 "[WebSocketServer] Received binary data from {}: {} bytes",
//                 client_id,
//                 data.len()
//             );
//             // Convert binary to hex string for the callback
//             let hex_string = data
//                 .iter()
//                 .map(|b| format!("{:02x}", b))
//                 .collect::<String>();

//             match call_callback(
//                 &on_message.0,
//                 &on_message.1,
//                 &on_message.2,
//                 vec![
//                     Value::String(client_id.to_string()),
//                     Value::String(format!("binary:{}", hex_string)),
//                 ],
//                 functions,
//                 native_functions,
//             ) {
//                 Ok(response) => {
//                     if !send_response(write, response).await {
//                         return false;
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("[WebSocketServer] onMessage error: {}", e);
//                 }
//             }
//         }
//         Message::Ping(data) => {
//             // Automatically respond to pings with pongs
//             if let Err(e) = write.send(Message::Pong(data)).await {
//                 eprintln!("[WebSocketServer] Failed to send pong: {}", e);
//                 return false;
//             }
//         }
//         Message::Pong(_) => {
//             // Pong received, no action needed
//         }
//         Message::Close(_) => {
//             println!("[WebSocketServer] Client closing: {}", client_id);
//             return false;
//         }
//         Message::Frame(_) => {
//             // Raw frames are not typically handled
//         }
//     }
//     true
// }

// async fn send_response(
//     write: &mut futures_util::stream::SplitSink<
//         tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
//         Message,
//     >,
//     response: Value,
// ) -> bool {
//     if let Value::String(response_text) = response {
//         if !response_text.is_empty() {
//             if let Err(e) = write.send(Message::Text(response_text)).await {
//                 eprintln!("[WebSocketServer] Failed to send response: {}", e);
//                 return false;
//             }
//         }
//     }
//     true
// }

// fn call_callback(
//     _name: &str,
//     params: &[String],
//     body: &[crate::parser::Stmt],
//     arg_values: Vec<Value>,
//     functions: &std::collections::HashMap<String, (Vec<String>, Vec<crate::parser::Stmt>)>,
//     native_functions: &std::collections::HashMap<String, crate::interpreter::NativeFn>,
// ) -> Result<Value, String> {
//     use crate::interpreter::{ControlFlow, Interpreter};

//     if params.len() != arg_values.len() {
//         return Err(format!(
//             "Callback expects {} arguments, got {}",
//             params.len(),
//             arg_values.len()
//         ));
//     }

//     // Create a new interpreter for the callback
//     let mut callback_interpreter = Interpreter::new();
//     callback_interpreter.env.functions = functions.clone();
//     callback_interpreter.env.native_functions = native_functions.clone();

//     // Set callback parameters
//     for (param, value) in params.iter().zip(arg_values.iter()) {
//         callback_interpreter
//             .env
//             .set_variable(param.clone(), value.clone());
//     }

//     // Execute the callback body
//     for stmt in body {
//         match callback_interpreter.execute_statement(stmt)? {
//             ControlFlow::Return(value) => {
//                 return Ok(value);
//             }
//             ControlFlow::None => continue,
//         }
//     }

//     Ok(Value::Void)
// }
