use std::{
  fs::{File, OpenOptions},
  io::{self, Read as _, Write as _},
};

use serde::{de::DeserializeOwned, Serialize};

pub fn read_appended_structs_from_file<T: DeserializeOwned + Serialize>(
  file_path: &str,
) -> io::Result<Vec<T>> {
  let mut file = File::open(file_path)?;
  let mut buffer = Vec::new();
  file.read_to_end(&mut buffer)?;

  let mut structs = Vec::new();
  let mut offset = 0;

  while offset < buffer.len() {
    let next_struct: T = bincode::deserialize(&buffer[offset..])
      .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to deserialize"))?;
    structs.push(next_struct);

    offset += (bincode::serialized_size(&structs.last().unwrap()).unwrap()) as usize;
  }

  Ok(structs)
}

pub fn append_struct_to_file<T: Serialize>(file_path: &str, value: &T) -> io::Result<()> {
  let serialized_data = bincode::serialize(value)
    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to serialize"))?;

  let mut file = OpenOptions::new()
    .create(true) // Create the file if it doesn't exist
    .append(true) // Open in append mode
    .open(file_path)?;

  file.write_all(&serialized_data)?; // Write serialized data to the file
  Ok(())
}

use std::fs::{self, DirEntry};

use chrono::{DateTime, NaiveDateTime, Utc};

use super::DATE_FMT;

pub fn files_in_dir(dir_path: &str) -> Result<Vec<DirEntry>, std::io::Error> {
  let entries = fs::read_dir(dir_path)?; // Read the directory
  let count = entries
    .filter_map(Result::ok) // Filter out errors
    .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or(false)) // Filter for files
    .collect(); // Count the files
  Ok(count)
}

pub fn sort_snapshot_files(files: &mut [DirEntry]) {
  files.sort_by(|a, b| {
    let file_name_date_a = &a.file_name().to_str().unwrap().to_string()[0..19];
    let naive_date_a = NaiveDateTime::parse_from_str(file_name_date_a, DATE_FMT).unwrap();
    let utc_datetime_a: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_date_a, Utc);

    let file_name_date_b = &b.file_name().to_str().unwrap().to_string()[0..19];
    let naive_date_b = NaiveDateTime::parse_from_str(file_name_date_b, DATE_FMT).unwrap();
    let utc_datetime_b: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_date_b, Utc);

    utc_datetime_a.cmp(&utc_datetime_b)
  });
}
