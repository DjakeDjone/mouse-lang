pub mod sleep;
pub mod socket_server;

pub struct StdFn {
    name: String,
    args: Vec<String>,
    return_type: String,
}
