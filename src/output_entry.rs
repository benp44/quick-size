use std::cmp::PartialEq;

pub struct OutputEntry
{
    pub file_name: String,
    pub is_directory: bool,
    pub file_size: usize,
    pub file_size_string: String,
    pub file_size_readable: String,
    pub is_fully_scanned: bool,
}

impl PartialEq<OutputEntry> for OutputEntry
{
    fn eq(&self, other: &Self) -> bool
    {
        self.file_name == other.file_name
    }
}
