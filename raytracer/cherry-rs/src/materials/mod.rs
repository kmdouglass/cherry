//! Interface to external materials data.

use anyhow::Result;
use dirs::data_local_dir;
use lib_ria::Store;

const FOLDERNAME: &str = "cherry";
const FILENAME: &str = "materials.db";

/// Load materials from the local data directory.
/// 
/// The materials are stored in a bitcode-encoded file in the local data directory.
pub fn load_materials() -> Result<Store> {
    let data_dir = data_local_dir().ok_or(anyhow::anyhow!("Cannot find local data directory"))?;
    let filename = data_dir.join(FOLDERNAME).join(FILENAME);
    let file = std::fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    let store = Store::from_reader(reader)?;
    Ok(store)
}
