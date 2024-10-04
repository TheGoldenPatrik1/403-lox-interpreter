use std::io;
use std::fs::{File, OpenOptions};
use std::io::{Write};

pub fn write_output(file_name: &str, message: &str) -> io::Result<()> {
    // If the file_name is empty, write to stdout, otherwise, write to the specified file.
    if file_name.is_empty() {
        let stdout = io::stdout();  // Get stdout
        let mut handle = stdout.lock();  // Lock stdout for writing
        writeln!(handle, "{}", message)?;  // Write the message to stdout
    } else {
        // Open the file in append mode, creating it if it doesn't exist
        let file = OpenOptions::new()
            .append(true)  // Enable append mode
            .create(true)  // Create the file if it doesn't exist
            .open(file_name)?;  // Open the file
        let mut handle = file;  // Use the file handle
        writeln!(handle, "{}", message)?;  // Write the message to the file
    }
    Ok(())
}
