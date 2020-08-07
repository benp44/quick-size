use super::directory_entry::DirectoryEntry;
use super::print_entry::PrintEntry;
use colored::*;
use humansize::file_size_opts::{FileSizeOpts, FixedAt, Kilo};
use humansize::FileSize;
use sorted_list::SortedList;
use std::cmp;
use std::io;
use std::vec::Vec;
use term_size;

const TOTAL_NAME: &str = ".";

pub fn print_directory_entries(directory_entries: &Vec<DirectoryEntry>) -> io::Result<()>
{
    let mut longest_name = 0;
    let mut longest_size = 0;
    let mut longest_size_readable = 0;

    let mut total_size = 0;
    let mut total_is_fully_scanned = true;

    let mut output_data_entries: SortedList<usize, PrintEntry> = SortedList::new();

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

    for directory_entry in directory_entries
    {
        let name = directory_entry.file_name.to_string();
        let size = directory_entry.file_size;
        let size_readable = directory_entry.file_size.file_size(&human_readable_options).unwrap();

        let entry = PrintEntry {
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

    let full_graph_width = get_graph_width();

    let show_total_size = true;
    
    if show_total_size {
        let total_size_string = total_size.to_string();
        let total_size_readable = total_size.file_size(&human_readable_options).unwrap();

        longest_name = cmp::max(longest_name, TOTAL_NAME.len());
        longest_size = cmp::max(longest_size, total_size.to_string().len());
        longest_size_readable = cmp::max(longest_size_readable, total_size_readable.len());

        let entry = build_output_string(
            &TOTAL_NAME,
            true,
            &total_size_string,
            &total_size_readable,            
            total_is_fully_scanned,
            longest_name,
            longest_size,
            longest_size_readable,
        );

        print!("{}", entry);
        println!("{}", build_graph_string(total_size, total_size, full_graph_width, '▕', '─', '▏'));        
    }

    for (_, print_entry) in output_data_entries.iter().rev()
    {
        let entry = build_output_string(
            &print_entry.file_name,
            print_entry.is_directory,
            &print_entry.file_size_string,
            &print_entry.file_size_readable,
            print_entry.is_fully_scanned,
            longest_name,
            longest_size,
            longest_size_readable,
        );

        print!("{}", entry);
        println!("{}", build_graph_string(print_entry.file_size, total_size, full_graph_width, '▕', '─', '▏'));
    }

    Ok(())
}

fn get_graph_width() -> usize
{
    let result = term_size::dimensions();
    match result
    {
        Some((terminal_width, _)) => (terminal_width / 3) as usize,
        None => 20 as usize,
    }
}

fn build_graph_string(
    file_size: usize,
    total_size: usize,
    print_width: usize,
    start_char: char,
    line_char: char,
    end_char: char,
) -> String
{
    let proportion = file_size as f64 / total_size as f64;
    let length_f = proportion * print_width as f64;
    let length = length_f.floor() as usize;

    let mut graph = String::new();

    graph.push(start_char);

    for _ in 0..length
    {
        graph.push(line_char);
    }

    if length != 0
    {
        graph.push(end_char);
    }

    graph
}

fn build_output_string(
    file_name: &str,
    is_directory: bool,
    file_size_string: &str,
    file_size_readable: &str,
    is_fully_scanned: bool,
    print_width_name: usize,
    print_width_size: usize,
    print_width_size_readable: usize,
) -> String
{
    let mut output_line = String::new();

    if is_directory
    {
        output_line += &format!("{:name_width$}", file_name, name_width = print_width_name).yellow().bold().to_string();
    }
    else
    {
        output_line += &format!("{:name_width$}", file_name, name_width = print_width_name);
    }

    output_line += " ";

    output_line += &format!("{:>size_width$}", file_size_string, size_width = print_width_size);
    output_line += " ";

    if is_fully_scanned
    {
        output_line += " ";
    }
    else
    {
        output_line += &"?".red().to_string();
    }

    output_line += " ";

    output_line += &format!("{:>size_readable_width$}", file_size_readable, size_readable_width = print_width_size_readable);

    output_line
}

// #[cfg(test)]
// mod test_printing_entries
// {
//     use super::print_directory_entries;
//     use colored::*;

//     #[test]
//     fn basic_directory() 
//     {
//         let results = Vec<DirectoryEntry>::New();

//         print_directory_entries
//     }
// }

#[cfg(test)]
mod test_printing_entry 
{
    use super::build_output_string;
    use colored::*;

    #[test]
    fn basic_directory() 
    {
        let entry = build_output_string(
            &"hello",
            true,
            &"1024",
            &"1024 B",
            true,
            10,
            10,
            10,
        );

        assert_eq!(format!("{} {} {} {}", "hello     ".yellow().bold(),"      1024", " ", "    1024 B"), entry);
    }

    #[test]
    fn uncertain_size() 
    {
        let entry = build_output_string(
            &"a",
            false,
            &"1",
            &"1 B",
            false,
            1,
            1,
            3,
        );

        assert_eq!(format!("{} {} {} {}", "a","1", "?".red(), "1 B"), entry);
    }
}

#[cfg(test)]
mod test_printing_graphs 
{
    use super::build_graph_string;

    #[test]
    fn empty_graph() 
    {
        assert_eq!("▕", build_graph_string(0, 10, 10, '▕', '─', '▏'));
    }

    #[test]
    fn no_width() 
    {
        assert_eq!("▕", build_graph_string(5, 10, 0, '▕', '─', '▏'));
    }

    #[test]
    fn full_graph() 
    {
        assert_eq!("▕──────────▏", build_graph_string(10, 10, 10, '▕', '─', '▏'));
    }

    #[test]
    fn half_graph() 
    {
        assert_eq!("▕─────▏", build_graph_string(5, 10, 10, '▕', '─', '▏'));
    }

    #[test]
    fn data_floored() 
    {
        assert_eq!("▕────▏", build_graph_string(21, 100, 20, '▕', '─', '▏'));
    }
}
