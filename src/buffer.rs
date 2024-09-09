use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub struct Buffer {
    pub lines: Vec<String>,
    pub path: String,
    pub command: String,
}

impl Buffer {
    pub fn new(path: String) -> io::Result<Self> {
        match File::open(&path) {
            Ok(mut file) => {
                let mut buf = String::new();
                file.read_to_string(&mut buf).unwrap();
                Ok(Self {
                    lines: buf.lines().map(|line| line.to_string()).collect(),
                    path: path,
                    command: String::new(),
                })
            }
            Err(_) => Ok(Self {
                lines: vec!["".to_string()],
                path: path,
                command: String::new(),
            }),
        }
    }
    pub fn save(&self) -> Result<(), io::Error> {
        match File::create(&self.path) {
            Ok(mut file) => {
                for line in self.lines.iter() {
                    if let Err(err) = file.write(format!("{line}\n").as_bytes()) {
                        return Err(err);
                    }
                }
            }
            Err(err) => return Err(err),
        }

        Ok(())
    }
}
