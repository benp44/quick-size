use std::time::Instant;
use std::vec::Vec;

mod directory_entry;
mod error_handler;
mod print_entry;
mod printer;
mod scanner;

fn main()
{
    let now = Instant::now();

    let mut directory_entries = Vec::new();
    let result = scanner::scan_current_directory(&mut directory_entries);

    if let Err(result_err) = result
    {
        error_handler::show_error(&result_err);
        return;
    }

    let result = printer::print_directory_entries(&directory_entries);

    if let Err(result_err) = result
    {
        error_handler::show_error(&result_err);
        return;
    }

    eprintln!("");
    eprintln!("Took {}ms", now.elapsed().as_millis());
}
