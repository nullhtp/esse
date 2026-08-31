//! Sparks, one JSON object per line in `sparks.jsonl`.

use std::path::PathBuf;

use crate::error::Result;
use crate::model::Spark;
use crate::store::{append_line, read_jsonl, DataDir};

pub struct SparkStore {
    path: PathBuf,
}

impl SparkStore {
    pub fn new(data: &DataDir) -> Self {
        SparkStore {
            path: data.sparks_path(),
        }
    }

    /// Captures a line of text. Blank input is not a spark: nothing is written
    /// and `None` comes back, so the caller can leave the input as it was.
    pub fn capture(&self, text: &str) -> Result<Option<Spark>> {
        if text.trim().is_empty() {
            return Ok(None);
        }
        let spark = Spark::new(text);
        self.append(&spark)?;
        Ok(Some(spark))
    }

    /// Appends one line; previously stored lines are untouched.
    pub fn append(&self, spark: &Spark) -> Result<()> {
        append_line(&self.path, &serde_json::to_string(spark)?)
    }

    /// Every spark, newest first.
    pub fn load_all(&self) -> Result<Vec<Spark>> {
        let mut sparks = read_jsonl::<Spark>(&self.path)?;
        // Reverse first, then sort by time: the sort is stable, so sparks
        // captured within the same second keep the file's order, newest last
        // line first.
        sparks.reverse();
        sparks.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(sparks)
    }
}
