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
