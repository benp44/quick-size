use super::directory_entry::DirectoryEntry;
use super::error_handler::{show_error, show_error_for_path};
use num_cpus;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::result::Result;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::vec::Vec;

const DEFAULT_MAX_THREAD_COUNT: usize = 4;

static GLOBAL_THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
static MAX_THREAD_COUNT: AtomicUsize = AtomicUsize::new(DEFAULT_MAX_THREAD_COUNT);

pub fn scan_current_directory(directory_entries: &mut Vec<DirectoryEntry>) -> io::Result<()>
{
    set_thread_count();

    let current_path = env::current_dir()?;

    for entry in fs::read_dir(current_path)?
    {
        let mut file_name = "?".to_string();
        let mut is_directory = false;
        let mut file_size = 0;
        let mut is_fully_scanned = false;

        if entry.is_ok()
        {
            let item_path = entry.unwrap().path();
            let name = item_path.file_name().unwrap().to_os_string();
            file_name = name.into_string().unwrap();

            let metadata = fs::metadata(&item_path);

            if metadata.is_ok()
            {
                is_directory = metadata.unwrap().file_type().is_dir();

                let result = get_size_of_item(&item_path);
                if result.is_ok()
                {
                    let (result_file_size, result_is_fully_scanned) = result.unwrap();
                    file_size = result_file_size;
                    is_fully_scanned = result_is_fully_scanned;
                }
                else
                {
                    show_error_for_path(&result.unwrap_err(), &item_path);
                }
            }
            else
            {
                show_error_for_path(&metadata.unwrap_err(), &item_path);
            }
        }
        else
        {
            show_error(&entry.unwrap_err());
        }

        let entry = DirectoryEntry {
            file_name,
            is_directory,
            file_size,
            is_fully_scanned,
        };

        directory_entries.push(entry);
    }

    Ok(())
}

fn set_thread_count()
{
    let cpu_count = num_cpus::get();
    MAX_THREAD_COUNT.store(cpu_count, Ordering::Relaxed);
}

fn get_size_of_file(file_path: &PathBuf) -> Result<usize, io::Error>
{
    let file_metadata = fs::metadata(file_path)?;

    Ok(file_metadata.len() as usize)
}

fn get_size_of_directory(file_path: &PathBuf) -> Result<(usize, bool), io::Error>
{
    let mut result_size = 0;
    let mut is_result_fully_scanned = true;

    let mut thread_handles: Vec<thread::JoinHandle<Result<(usize, bool), io::Error>>> = Vec::new();

    let result = fs::read_dir(&file_path)?;
    for directory_entry_result in result
    {
        match directory_entry_result 
        {
            Ok(directory_entry) => 
            {
                let directory_entry_path = directory_entry.path();
                let metadata_result = fs::symlink_metadata(&directory_entry_path);
        
                match metadata_result
                {
                    Ok(metadata) => 
                    {
                        let file_type = metadata.file_type();
                        if file_type.is_dir() && GLOBAL_THREAD_COUNT.load(Ordering::Relaxed) < MAX_THREAD_COUNT.load(Ordering::Relaxed)
                        {
                            GLOBAL_THREAD_COUNT.fetch_add(1, Ordering::Relaxed);
        
                            let handler: thread::JoinHandle<Result<(usize, bool), io::Error>> = thread::spawn(move || {
                                let result = get_size_of_item(&directory_entry_path);
        
                                if result.is_err()
                                {
                                    show_error_for_path(&result.as_ref().unwrap_err(), &directory_entry_path);
                                }
        
                                GLOBAL_THREAD_COUNT.fetch_sub(1, Ordering::Relaxed);
        
                                result
                            });
        
                            thread_handles.push(handler);
                        }
                        else
                        {
                            let result = get_size_of_item(&directory_entry_path);
                            if let Ok(inner_result) = result
                            {
                                let (size, is_fully_scanned) = inner_result;
                                result_size += size;
                                is_result_fully_scanned &= is_fully_scanned;
                            }
                            else
                            {
                                is_result_fully_scanned = false;
                            }
                        }
                    },
                    Err(error) => 
                    {
                        is_result_fully_scanned = false;
                        show_error_for_path(&error, &directory_entry_path);
                        continue;
                    }
                }        
            }
            Err(error) => 
            {
                is_result_fully_scanned = false;
                show_error(&&error);
                continue;
            }
        }
    }

    for thread_handle in thread_handles
    {
        let join_result = thread_handle.join();
        if let Ok(result) = join_result
        {
            if let Ok(inner_result) = result
            {
                let (size, is_fully_scanned) = inner_result;
                result_size += size;
                is_result_fully_scanned &= is_fully_scanned;
            }
            else
            {
                is_result_fully_scanned = false;
            }
        }
    }

    Ok((result_size, is_result_fully_scanned))
}

fn get_size_of_item(file_path: &PathBuf) -> Result<(usize, bool), io::Error>
{
    let mut result_size = 0;
    let mut is_result_fully_scanned = true;

    let metadata = fs::symlink_metadata(&file_path)?;
    let file_type = metadata.file_type();

    if !file_type.is_symlink()
    {
        if file_type.is_file()
        {
            result_size += get_size_of_file(file_path)?;
        }
        else if file_type.is_dir()
        {
            let (size, is_fully_scanned) = get_size_of_directory(file_path)?;
            result_size += size;
            is_result_fully_scanned &= is_fully_scanned;
        }
    }

    Ok((result_size, is_result_fully_scanned))
}
