use colored::*;
use std::io;
use std::path::PathBuf;

pub fn show_error(error: &io::Error)
{
    eprintln!("{} {}", "Error: ".red(), error.to_string());
}

pub fn show_error_for_path(error: &io::Error, file_path: &PathBuf)
{
    eprintln!("{} {} {:?}", "Error: ".red(), error.to_string(), file_path);
}
