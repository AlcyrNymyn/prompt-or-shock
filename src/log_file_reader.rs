use std::{
    fs::{File, read_dir},
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    time::SystemTime,
};

pub struct LogFileReader {
    log_file: PathBuf,
    file_reader: BufReader<File>,
    buffer: String,
}

impl LogFileReader {
    pub fn new() -> Self {
        let user_profile_dir = std::env::var("USERPROFILE").expect("%USERPROFILE% not set");
        let mut log_folder = PathBuf::from(&user_profile_dir);
        log_folder.push("AppData\\LocalLow\\VRChat\\VRChat");
        println!("Log Directory: {}", log_folder.to_str().unwrap());

        let dir = read_dir(log_folder).expect("Unable to read VRChat log folder");
        let mut max_file: Option<(PathBuf, SystemTime)> = None;
        for path in dir {
            let file = if let Ok(file) = path { file } else { continue };
            let path = file.path();
            if path
                .file_name()
                .map(|os_str| os_str.to_str())
                .flatten()
                .is_some_and(|p| p.starts_with("output_log_"))
            {
                let new_time = file.metadata().unwrap().modified().unwrap();
                if let Some((_, time)) = max_file.clone() {
                    if new_time > time {
                        max_file = Some((file.path(), new_time));
                    }
                } else {
                    max_file = Some((file.path(), new_time));
                }
            }
        }

        let log_file = max_file.expect("Could not find log file.").0;
        println!("Log File: {}", log_file.to_str().unwrap());

        let raw_file_reader = File::open(&log_file).expect("Could not open log file.");
        let mut file_reader = BufReader::new(raw_file_reader);
        file_reader
            .seek(SeekFrom::End(0))
            .expect("Seek to end of file failed");
        Self {
            log_file,
            file_reader,
            buffer: String::with_capacity(200),
        }
    }

    pub fn log_file_path(&self) -> &PathBuf {
        &self.log_file
    }

    pub fn read_line<'a>(&'a mut self) -> Option<&'a str> {
        self.buffer.clear();
        match self.file_reader.read_line(&mut self.buffer) {
            Ok(0) => None,
            Ok(len) => {
                if self.buffer.ends_with('\n') {
                    Some(self.buffer.trim())
                } else {
                    self.file_reader
                        .seek_relative(-(len as i64))
                        .expect("Seek to undo partial line failed.");
                    None
                }
            }
            Err(error) => {
                println!("Error: {error}");
                None
            }
        }
    }
}
