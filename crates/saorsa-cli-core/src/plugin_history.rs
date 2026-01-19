use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

const HISTORY_FILE: &str = "plugin_history.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginRunStats {
    pub successes: u64,
    pub failures: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
}

impl PluginRunStats {
    #[must_use]
    pub fn total_runs(&self) -> u64 {
        self.successes + self.failures
    }
}

#[derive(Debug, Clone)]
pub struct PluginHistory {
    path: Option<PathBuf>,
    records: HashMap<String, PluginRunStats>,
}

impl Default for PluginHistory {
    fn default() -> Self {
        Self {
            path: default_history_path(),
            records: HashMap::new(),
        }
    }
}

impl PluginHistory {
    /// Load plugin history from disk if available.
    #[must_use]
    pub fn load() -> Self {
        let mut history = Self::default();

        if let Some(path) = history.path.as_ref() {
            if let Ok(contents) = fs::read_to_string(path) {
                if let Ok(records) =
                    serde_json::from_str::<HashMap<String, PluginRunStats>>(&contents)
                {
                    history.records = records;
                }
            }
        }

        history
    }

    #[must_use]
    pub fn stats_for(&self, plugin_name: &str) -> Option<&PluginRunStats> {
        self.records.get(plugin_name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &PluginRunStats)> {
        self.records.iter()
    }

    pub fn record_success(&mut self, plugin_name: &str) -> io::Result<()> {
        self.record(plugin_name, true, None)
    }

    pub fn record_failure(
        &mut self,
        plugin_name: &str,
        message: impl Into<Option<String>>,
    ) -> io::Result<()> {
        self.record(plugin_name, false, message.into())
    }

    fn record(
        &mut self,
        plugin_name: &str,
        success: bool,
        message: Option<String>,
    ) -> io::Result<()> {
        let stats = self
            .records
            .entry(plugin_name.to_string())
            .or_insert_with(PluginRunStats::default);
        if success {
            stats.successes += 1;
        } else {
            stats.failures += 1;
        }
        stats.last_run = Some(Utc::now());
        stats.last_status = message;
        self.save()
    }

    fn save(&self) -> io::Result<()> {
        let path = match &self.path {
            Some(path) => path,
            None => return Ok(()),
        };

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(&self.records)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(path, data)
    }
}

fn default_history_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("saorsa-cli").join(HISTORY_FILE))
}
