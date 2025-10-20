pub mod sleep;
pub mod socket_server;
pub mod str_utils;

pub struct StdFn {
    name: String,
    args: Vec<String>,
    return_type: String,
}
