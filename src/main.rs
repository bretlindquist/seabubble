pub mod mcp;
mod ui;
mod core;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("  __");
    println!(" >(')____, ");
    println!("   (` =~~/  ");
    println!(" ^~^`---'~^~^~");
    println!("SeaTurtle V2 Engine Booting...");
    std::thread::sleep(std::time::Duration::from_millis(600));

    // Dummy call to ui::setup_terminal if it doesn't exist
    // Let's check if it exists in ui/mod.rs. It doesn't seem to.
    ui::setup_terminal()?;
    
    Ok(())
}
