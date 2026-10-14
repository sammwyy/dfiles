//! Standalone File Explorer application.

use dfiles::{
    cli::{self, Launch},
    show_window, WindowRequest,
};

fn main() -> Result<(), String> {
    match cli::parse(std::env::args_os().skip(1))? {
        Launch::Manager(path) => {
            show_window(WindowRequest::manager(path));
        }
        Launch::Portal => dfiles::portal::service::run()?,
        Launch::Help => println!("{}", cli::HELP),
        Launch::Version => println!("File Explorer {}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}
