use std::io;
use std::fs::File;
use std::io::{Write};

pub fn write_output(file_name: &str, message: &str) -> io::Result<()> {
    // If the file_name is empty, write to stdout, otherwise, write to the specified file.
    if file_name.is_empty() {
        let stdout = io::stdout();  // Get stdout
        let mut handle = stdout.lock();  // Lock stdout for writing
        writeln!(handle, "{}", message)?;  // Write the message to stdout
    } else {
        let file = File::create(file_name)?;  // Create or overwrite the file
        let mut handle = file;  // Use the file handle
        writeln!(handle, "{}", message)?;  // Write the message to the file
    }
    Ok(())
}