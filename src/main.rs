use std::cmp;
use std::env;
use std::fs;
use std::io;
use std::result::Result;
use std::vec::Vec;

use colour;
use humansize::{FileSize, file_size_opts as options};
use sorted_list::SortedList;
use term_size;

struct DirectoryEntry {
    file_name: String,
    file_type: fs::FileType,
    file_size: usize
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
    file_size_readable: String
}

impl cmp::PartialEq<OutputData> for OutputData {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
    }
}

fn show_error(error : io::Error) {
    colour::red!("Error: ");
    println!("{}", error.to_string());
}

fn get_subdirectory_size(path : &str) -> Result<usize, io::Error> {

    let mut subdirectory_size = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        let path_str = path.to_str().unwrap();

        if metadata.is_file() {
            subdirectory_size += metadata.len() as usize;
        } else if metadata.is_dir() {
            subdirectory_size += get_subdirectory_size(path_str)?;
        }
    }

    Ok(subdirectory_size)
}

fn get_full_graph_width(minimum_space : usize) -> usize {

    let (terminal_width, _) = term_size::dimensions().unwrap();
    let graph_width = ((terminal_width as usize / 3) - minimum_space) as usize;
    
    graph_width
}

fn build_graph(file_size : &usize, total_size : &usize, full_graph_width : &usize) -> String {
    let proportion = *file_size as f64 / *total_size as f64;
    let length_f = proportion * *full_graph_width as f64;
    let length = length_f.floor() as usize;

    let mut graph = String::from("");

    for _ in 0..length {
        graph.push('█');
    }

    if length > 0 {
        graph.push(' ');
    }

    graph
}

fn scan_current_directory() -> io::Result<()> {
    let path = env::current_dir()?;

    let mut directory_entries: Vec<DirectoryEntry> = Vec::new();
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        let metadata = fs::metadata(&path)?;

        let mut file_size = 0;

        if metadata.is_file() {
            file_size = metadata.len() as usize;
        } 
        else if metadata.is_dir() {
            file_size = get_subdirectory_size(name)?;
        }
        else {
            continue
        }

        let entry = DirectoryEntry{
            file_name: name.to_string(), 
            file_type: metadata.file_type(),
            file_size: file_size
        };

        directory_entries.push(entry);
    }

    // Process

    let mut output_data_entries : SortedList<usize, OutputData> = SortedList::new();
    let mut longest_name = 0;
    let mut longest_size = 0;
    let mut longest_size_readable = 0;
    let mut total_size = 0;

    for directory_entry in directory_entries {

        let name = directory_entry.file_name;
        let size = directory_entry.file_size;
        let size_readable = directory_entry.file_size.file_size(options::CONVENTIONAL).unwrap();

        let entry = OutputData{
            file_name: name,
            file_type: directory_entry.file_type,            
            file_size: size,
            file_size_string: size.to_string(),
            file_size_readable: size_readable
        };
    
        longest_name = cmp::max(longest_name, entry.file_name.len());
        longest_size = cmp::max(longest_size, entry.file_size_string.len());
        longest_size_readable = cmp::max(longest_size_readable, entry.file_size_readable.len());
        total_size += size;

        output_data_entries.insert(size, entry);
    }
    
    // Print

    let full_graph_width = get_full_graph_width(longest_name + longest_size + longest_size_readable);
    let show_graph = full_graph_width > 20;

    for (_, output_entry) in output_data_entries.iter().rev() {

        print!("{:name_width$} {:size_width$} {}{:size_readable_width$}", 
                    output_entry.file_name, 
                    output_entry.file_size,
                    build_graph(&output_entry.file_size, &total_size, &full_graph_width),
                    output_entry.file_size_readable,
                    name_width = longest_name,
                    size_width = longest_size,
                    size_readable_width = longest_size_readable);

        // if show_graph {

        //     if output_entry.file_type.is_dir() {
        //         colour::blue!(" {}", build_graph(&output_entry.file_size, &total_size, &full_graph_width));
        //     }
        //     else if output_entry.file_type.is_file() {
        //         print!(" {}", build_graph(&output_entry.file_size, &total_size, &full_graph_width));
        //     }
        // }

        println!("");

        // if output_entry.file_type.is_file() {
        //     print!("{:name_width$} {:20} {:20}", output_entry.file_name, file_size, file_size.file_size(options::CONVENTIONAL).unwrap(), name_width = longest_name);

        //     if show_graph {
        //         print!(" {}", build_graph(file_size, &total_size, &full_graph_width));
        //     }

        //     println!("");
        // } 
        // else if output_entry.file_type.is_dir() {
        //     colour::yellow!("{:name_width$} {:20} {:20}", output_entry.file_name, file_size, file_size.file_size(options::CONVENTIONAL).unwrap(), name_width = longest_name);

        //     if show_graph {
        //         colour::yellow!(" {}", build_graph(file_size, &total_size, &full_graph_width));
        //     }

        //     colour::yellow_ln!("");
        // }
    }

    Ok(())
}

fn main() {
    let result = scan_current_directory();

    match result {
        Ok(_v) => (),
        Err(e) => show_error(e),
    }
}

