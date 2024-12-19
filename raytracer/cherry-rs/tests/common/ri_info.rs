use std::io::Read;

use anyhow::Result;
use lib_ria::Store;

pub fn load_store() -> Result<Store> {
    let filename = std::path::PathBuf::from("tests/data/rii.db");
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    let data = reader.bytes().collect::<Result<Vec<u8>, _>>()?;
    
    let store: Store = bitcode::deserialize(&data)?;
    Ok(store)
}
