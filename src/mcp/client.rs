pub struct McpClientImpl;

impl McpClientImpl {
    pub async fn call_tool(&self, _name: &str, _args: String) -> Result<String, std::io::Error> {
        Ok("mock".into())
    }
}
