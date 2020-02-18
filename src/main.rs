use std::cmp;
use std::env;
use std::fs;
use std::io;
use std::path;
use std::result::Result;
use std::vec::Vec;

use colored::*;
use humansize::{file_size_opts as options, FileSize};
use sorted_list::SortedList;
use term_size;

struct DirectoryEntry {
    file_name: String,
    file_type: fs::FileType,
    file_size: usize,
    is_fully_scanned: bool,
}

impl cmp::PartialEq<DirectoryEntry> for DirectoryEntry {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
    }
}

struct OutputData {
    file_name: String,
    file_type: fs::FileType,
    file_size: usize,
    file_size_string: String,
    file_size_readable: String,
    is_fully_scanned: bool,
}

impl cmp::PartialEq<OutputData> for OutputData {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
    }
}

fn show_error(error: io::Error, additional_message: &str) {
    print!("{}", "Error: ".red());
    println!("{} {}", error.to_string(), additional_message);
}

fn get_file_size(file_path: &str) -> Result<(usize, bool), io::Error> {
    let mut result_size = 0;
    let mut is_result_fully_scanned = true;

    let path_wrapper = path::Path::new(file_path);

    if path_wrapper.is_file() {
        let file_metadata = fs::metadata(path_wrapper)?;
        result_size += file_metadata.len() as usize;
    } else if path_wrapper.is_dir() {
        let result = fs::read_dir(file_path);

        if result.is_ok() {
            for contained_file in result.unwrap() {
                let contained_file = contained_file?;
                let contained_file_path = contained_file.path();
                let contained_file_path_str = contained_file_path.to_str().unwrap();
                let file_type = contained_file.file_type()?;

                if file_type.is_symlink() == false {
                    if file_type.is_file() {
                        let file_metadata = fs::metadata(&contained_file_path)?;
                        result_size += file_metadata.len() as usize;
                    } else if file_type.is_dir() {
                        let (size, is_fully_scanned) = get_file_size(contained_file_path_str)?;
                        result_size += size;
                        is_result_fully_scanned &= is_fully_scanned;
                    }
                }
            }
        } else {
            is_result_fully_scanned = false;
            show_error(result.err().unwrap(), file_path);
        }
    }

    Ok((result_size, is_result_fully_scanned))
}

fn get_graph_width() -> usize {
    let result = term_size::dimensions();
    match result {
        Some((terminal_width, _)) => (terminal_width / 3) as usize,
        None => 20 as usize,
    }
}

fn build_graph(file_size: usize, total_size: usize, full_graph_width: usize) -> String {
    let proportion = file_size as f64 / total_size as f64;
    let length_f = proportion * full_graph_width as f64;
    let length = length_f.floor() as usize;

    let mut graph = String::from("▕");

    for _ in 0..length {
        graph.push('█');
    }

    graph
}

fn scan_current_directory(directory_entries: &mut Vec<DirectoryEntry>) -> io::Result<()> {
    let path = env::current_dir()?;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        let metadata = fs::metadata(&path)?;

        let (file_size, is_fully_scanned) = get_file_size(name)?;

        let entry = DirectoryEntry {
            file_name: name.to_string(),
            file_type: metadata.file_type(),
            file_size: file_size,
            is_fully_scanned: is_fully_scanned,
        };

        directory_entries.push(entry);
    }

    Ok(())
}

fn print_directory_entries(directory_entries: &Vec<DirectoryEntry>) -> io::Result<()> {
    let mut longest_name = 0;
    let mut longest_size = 0;
    let mut longest_size_readable = 0;
    let mut total_size = 0;

    let mut output_data_entries: SortedList<usize, OutputData> = SortedList::new();

    for directory_entry in directory_entries {
        let name = directory_entry.file_name.to_string();
        let size = directory_entry.file_size;
        let size_readable = directory_entry.file_size.file_size(options::CONVENTIONAL).unwrap();

        let entry = OutputData {
            file_name: name,
            file_type: directory_entry.file_type,
            file_size: size,
            file_size_string: size.to_string(),
            file_size_readable: size_readable,
            is_fully_scanned: directory_entry.is_fully_scanned,
        };

        longest_name = cmp::max(longest_name, entry.file_name.len());
        longest_size = cmp::max(longest_size, entry.file_size_string.len());
        longest_size_readable = cmp::max(longest_size_readable, entry.file_size_readable.len());
        total_size += size;

        output_data_entries.insert(size, entry);
    }

    let full_graph_width = get_graph_width();

    for (_, output_entry) in output_data_entries.iter().rev() {
        let mut output_line = String::new();

        output_line += &format!("{:name_width$} ", output_entry.file_name, name_width = longest_name);
        output_line += &format!("{:>size_width$} ", output_entry.file_size, size_width = longest_size);

        if output_entry.is_fully_scanned {
            output_line += " ";
        } else {
            output_line += "?";
        }

        output_line += &format!("{} ", build_graph(output_entry.file_size, total_size, full_graph_width));
        output_line += &format!("{:size_readable_width$}", output_entry.file_size_readable, size_readable_width = longest_size_readable);

        if output_entry.file_type.is_dir() {
            print!("{}", output_line.blue());
        } else {
            print!("{}", output_line);
        }

        println!("");
    }

    Ok(())
}

fn main() {
    let mut directory_entries: Vec<DirectoryEntry> = Vec::new();
    let result = scan_current_directory(&mut directory_entries);

    if result.is_err() {
        show_error(result.unwrap_err(), "");
        return;
    }

    let result = print_directory_entries(&directory_entries);

    if result.is_err() {
        show_error(result.unwrap_err(), "");
        return;
    }
}
