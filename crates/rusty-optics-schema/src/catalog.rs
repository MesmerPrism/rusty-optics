use rusty_optics_model::optics_schema_ids;
use serde::Serialize;

/// Compact schema catalog.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SchemaCatalog {
    /// Catalog schema version.
    pub schema_version: u32,
    /// Catalog entries.
    pub schemas: Vec<SchemaEntry>,
}

/// One schema entry.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SchemaEntry {
    /// Schema identifier.
    pub schema_id: &'static str,
    /// Owning Rusty layer.
    pub owner: &'static str,
    /// Current stability status.
    pub status: &'static str,
}

/// Builds the current schema catalog.
#[must_use]
pub fn build_catalog() -> SchemaCatalog {
    SchemaCatalog {
        schema_version: 1,
        schemas: optics_schema_ids()
            .into_iter()
            .map(|schema_id| SchemaEntry {
                schema_id,
                owner: "rusty-optics",
                status: "foundation",
            })
            .collect(),
    }
}

/// Serializes the schema catalog as stable pretty JSON.
pub fn catalog_json() -> Result<String, serde_json::Error> {
    let mut json = serde_json::to_string_pretty(&build_catalog())?;
    json.push('\n');
    Ok(json)
}
