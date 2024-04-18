use std::fs::File;
use std::io::{BufRead, BufReader};
/// Reads a file line by line
pub struct FileLines {
    reader: BufReader<File>,
}

impl FileLines {
    /// Creates a new FileLines instance
    pub fn new(file_path: &str) -> std::io::Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        Ok(Self { reader })
    }

    /// Reads the next line of the file
    pub fn next_line(&mut self) -> Option<Vec<u8>> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line);

        match bytes_read {
            Ok(0) => {
                return None;
            }
            Err(_) => {
                return None;
            }
            Ok(_) => {}
        };

        line.pop();

        let mut result = Vec::new();
        for char in line.split(',') {
            let num: u8 = match char.parse() {
                Ok(num) => num,
                Err(_) => {
                    continue;
                }
            };
            result.push(num);
        }
        Some(result)
    }
}

#[cfg(test)]

mod file_lines_tests {
    use super::*;

    #[test]
    fn test_file_lines() {
        let mut file_lines = FileLines::new("././storage/testing").unwrap();
        let line = file_lines.next_line();
        assert!(line.is_none());
    }
}
