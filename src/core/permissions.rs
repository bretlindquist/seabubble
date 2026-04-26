pub fn requires_approval(tool_name: &str, _arguments: &str) -> bool {
    if tool_name.starts_with("read_")
        || tool_name.starts_with("list_")
        || tool_name.starts_with("search_")
    {
        return false;
    }

    match tool_name {
        "echo" | "cat" | "ls" | "grep" | "rg" => false,
        _ => true,
    }
}
