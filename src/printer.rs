use super::directory_entry::DirectoryEntry;
use super::output_entry::OutputEntry;
use colored::*;
use humansize::file_size_opts::{FileSizeOpts, FixedAt, Kilo};
use humansize::FileSize;
use sorted_list::SortedList;
use std::cmp;
use std::io;
use std::vec::Vec;
use term_size;

const TOTAL_NAME: &str = "Total";

pub fn print_directory_entries(directory_entries: &Vec<DirectoryEntry>) -> io::Result<()>
{
    let mut longest_name = 0;
    let mut longest_size = 0;
    let mut longest_size_readable = 0;

    let mut total_size = 0;
    let mut total_is_fully_scanned = true;

    let mut output_data_entries: SortedList<usize, OutputEntry> = SortedList::new();

    let human_readable_options = FileSizeOpts {
        divider: Kilo::Binary,
        units: Kilo::Decimal,
        decimal_places: 1,
        decimal_zeroes: 0,
        fixed_at: FixedAt::No,
        long_units: false,
        space: true,
        suffix: "",
        allow_negative: true,
    };

    for directory_entry in directory_entries {
        let name = directory_entry.file_name.to_string();
        let size = directory_entry.file_size;
        let size_readable = directory_entry.file_size.file_size(&human_readable_options).unwrap();

        let entry = OutputEntry {
            file_name: name,
            is_directory: directory_entry.is_directory,
            file_size: size,
            file_size_string: size.to_string(),
            file_size_readable: size_readable,
            is_fully_scanned: directory_entry.is_fully_scanned,
        };

        longest_name = cmp::max(longest_name, entry.file_name.len());
        longest_size = cmp::max(longest_size, entry.file_size_string.len());
        longest_size_readable = cmp::max(longest_size_readable, entry.file_size_readable.len());

        output_data_entries.insert(size, entry);

        total_size += size;
        total_is_fully_scanned &= directory_entry.is_fully_scanned;
    }

    let total_size_readable = total_size.file_size(&human_readable_options).unwrap();

    longest_name = cmp::max(longest_name, TOTAL_NAME.len());
    longest_size = cmp::max(longest_size, total_size.to_string().len());
    longest_size_readable = cmp::max(longest_size_readable, total_size_readable.len());

    let full_graph_width = get_graph_width();

    print_summary_entry(
        total_is_fully_scanned,
        total_size,
        &total_size_readable,
        full_graph_width,
        longest_name,
        longest_size,
        longest_size_readable,
    );

    for (_, output_entry) in output_data_entries.iter().rev() {
        print_entry(
            output_entry.is_directory,
            &output_entry.file_name,
            &output_entry.file_size_string,
            output_entry.is_fully_scanned,
            output_entry.file_size,
            &output_entry.file_size_readable,
            total_size,
            full_graph_width,
            longest_name,
            longest_size,
            longest_size_readable,
        );
    }

    Ok(())
}

fn get_graph_width() -> usize
{
    let result = term_size::dimensions();
    match result {
        Some((terminal_width, _)) => (terminal_width / 3) as usize,
        None => 20 as usize,
    }
}

fn build_graph(file_size: usize, total_size: usize, full_graph_width: usize, start_char: char, line_char: char, end_char: char) -> String
{
    let proportion = file_size as f64 / total_size as f64;
    let length_f = proportion * full_graph_width as f64;
    let length = length_f.floor() as usize;

    let mut graph = String::new();

    graph.push(start_char);

    for _ in 0..length {
        graph.push(line_char);
    }

    if length != 0 {
        graph.push(end_char);
    }

    graph
}

fn print_summary_entry(is_fully_scanned: bool, total_size: usize, total_size_readable: &str, full_graph_width: usize, longest_name: usize, longest_size: usize, longest_size_readable: usize)
{
    let mut output_line = String::new();

    output_line += &format!("{:name_width$} ", TOTAL_NAME, name_width = longest_name).bold().to_string();
    output_line += &format!("{:>size_width$} ", total_size, size_width = longest_size);

    if is_fully_scanned {
        output_line += " ";
    } else {
        output_line += &"?".red().to_string();
    }

    output_line += &format!("{:>size_readable_width$}", total_size_readable, size_readable_width = longest_size_readable);
    output_line += &format!("{} ", build_graph(total_size, total_size, full_graph_width, '▕', '━', '▏'));

    println!("{}", output_line.white().to_string());
}

fn print_entry(
    is_directory: bool,
    file_name: &str,
    file_size_string: &str,
    is_fully_scanned: bool,
    file_size: usize,
    file_size_readable: &str,
    total_size: usize,
    full_graph_width: usize,
    longest_name: usize,
    longest_size: usize,
    longest_size_readable: usize,
)
{
    let mut output_line = String::new();

    if is_directory {
        output_line += &format!("{:name_width$} ", file_name, name_width = longest_name).yellow().bold().to_string();
    } else {
        output_line += &format!("{:name_width$} ", file_name, name_width = longest_name);
    }

    output_line += &format!("{:>size_width$} ", file_size_string, size_width = longest_size);

    if is_fully_scanned {
        output_line += " ";
    } else {
        output_line += &"?".red().to_string();
    }

    output_line += &format!("{:>size_readable_width$}", file_size_readable, size_readable_width = longest_size_readable);
    output_line += &format!("{} ", build_graph(file_size, total_size, full_graph_width, '▕', '─', '▏'));

    println!("{}", output_line);
}
