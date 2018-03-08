use std::path::{Path, PathBuf};
use parity_wasm;
use parity_wasm::elements;
use parity_wasm::elements::{ExportEntry, Internal, Module, Section};

#[derive(Debug)]
pub enum Error {
    DeserializeWasmError(elements::Error),
    MissingTableSection,
    InvalidTableSectionCount(usize),
    MissingExportSection,
    SerializeWasmError(elements::Error),
}

fn export_table(module: &mut Module) -> Result<(), Error> {
    let table_index: u32 = 0;
    // Make sure there's only one table to export, which will be at table index 0
    {
        let table_section = module.table_section().ok_or(Error::MissingTableSection)?;
        let entries = table_section.entries();
        if entries.len() != 1 {
            return Err(Error::InvalidTableSectionCount(entries.len()));
        }
    }

    let exports = module
        .sections_mut()
        .iter_mut()
        .filter_map(|section| match *section {
            Section::Export(ref mut e) => Some(e),
            _ => None,
        })
        .next()
        .ok_or(Error::MissingExportSection)?;
    // Add an export for the table at index 0 to the exports section
    exports.entries_mut().push(ExportEntry::new(
        "table".to_string(),
        Internal::Table(table_index),
    ));

    Ok(())
}

pub fn process(wasm_file: &Path) -> Result<PathBuf, Error> {
    let mut module = parity_wasm::deserialize_file(wasm_file).map_err(Error::DeserializeWasmError)?;

    export_table(&mut module)?;

    parity_wasm::serialize_to_file(wasm_file, module).map_err(Error::SerializeWasmError)?;
    Ok(wasm_file.to_path_buf())
}
