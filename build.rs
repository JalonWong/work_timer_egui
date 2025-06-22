use std::{env, io};

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        use winres::WindowsResource;
        WindowsResource::new()
            .set_icon("assets/timer.ico")
            .compile()?;
    }
    Ok(())
}
