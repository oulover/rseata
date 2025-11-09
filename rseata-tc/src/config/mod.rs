use std::env;
fn get_env(key: &str) -> Option<String> {
    env::var(key).ok()
}
pub fn get_env_web_server_ip() -> String {
    get_env("RSEATA_WEB_SERVER_IP").unwrap_or(String::from("0.0.0.0"))
}
pub fn get_env_web_server_prot() -> String {
    get_env("RSEATA_WEB_SERVER_PROT").unwrap_or(String::from("3000"))
}
pub fn get_env_rseata_traces_exporter() -> String {
    get_env("RSEATA_TRACES_EXPORTER").unwrap_or(String::from("FMT"))
}


pub fn get_env_grpc_server_ip() -> String {
    get_env("RSEATA_GRPC_SERVER_IP").unwrap_or(String::from("0.0.0.0"))
}
pub fn get_env_grpc_server_prot() -> String {
    get_env("RSEATA_GRPC_SERVER_IP").unwrap_or(String::from("9811"))
}
