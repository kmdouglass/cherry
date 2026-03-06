/// gen-rii-index: reads rii.db and writes a flat JSON array of material keys.
///
/// Usage: gen-rii-index --output <path>
///
/// The output file contains one JSON array of "shelf:book:page" strings,
/// matching the format expected by MaterialIndex::build_from_keys.
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let output_path = loop {
        match args.next().as_deref() {
            Some("--output") => {
                break args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--output requires a value"))?;
            }
            Some(flag) => {
                anyhow::bail!("Unknown argument: {flag}")
            }
            None => {
                break "assets/rii-index.json".to_string();
            }
        }
    };

    let db_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/rii.db");
    let data = std::fs::read(&db_path)
        .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", db_path.display()))?;

    let store: lib_ria::Store = bitcode::deserialize(&data)
        .map_err(|e| anyhow::anyhow!("Cannot deserialize rii.db: {e}"))?;

    let mut keys: Vec<String> = store.keys().cloned().collect();
    keys.sort();

    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let file = std::fs::File::create(&output_path)
        .map_err(|e| anyhow::anyhow!("Cannot create {output_path}: {e}"))?;
    serde_json::to_writer(file, &keys)?;

    println!("Wrote {} keys to {output_path}", keys.len());
    Ok(())
}
