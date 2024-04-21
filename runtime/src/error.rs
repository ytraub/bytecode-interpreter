pub fn dissasemble_error(msg: String) -> String {
    return format!("[DISSASEMBLE]: {}", msg);
}

pub fn runtime_error(msg: String) -> String {
    return format!("[RUNTIME]: {}", msg);
}

pub fn repl_error(msg: String) -> String {
    return format!("[REPL]: {}", msg);
}