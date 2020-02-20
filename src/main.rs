use std::time;
use std::vec::Vec;

mod error_handler;
mod output_entry;
mod directory_entry;
mod printer;
mod scanner;

fn main()
{
    let now = time::Instant::now();

    let mut directory_entries = Vec::new();
    let result = scanner::scan_current_directory(&mut directory_entries);

    if result.is_err() {
        error_handler::show_error(&result.unwrap_err());
        return;
    }

    let result = printer::print_directory_entries(&directory_entries);

    if result.is_err() {
        error_handler::show_error(&result.unwrap_err());
        return;
    }

    eprintln!("");
    eprintln!("Took {}ms", now.elapsed().as_millis());
}
