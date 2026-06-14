//! Output run-marker.
//!
//! File and report output is produced by the `Output*` pipeline steps (see
//! `fastqrab-steps/src/transformations/output/`). The only output-related
//! responsibility left here is the `.incompleted` run marker, which is written
//! before a run starts and removed on success so an interrupted run can be
//! detected.

use anyhow::{Context, Result, anyhow};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct OutputRunMarker {
    pub path: PathBuf,
    preexisting: bool,
}

impl OutputRunMarker {
    pub fn create(output_directory: &Path, prefix: &str) -> Result<Self> {
        let path = output_directory.join(format!("{prefix}.incompleted"));
        let prefix_parent = path
            .parent()
            .expect("Really expected a parent on a joined directory");
        if prefix_parent != output_directory {
            ex::fs::create_dir_all(prefix_parent).with_context(|| {
                format!(
                    "Could not create output (sub) directory for completion marker: {}",
                    prefix_parent.display()
                )
            })?;
        }
        let preexisting = std::fs::symlink_metadata(&path).is_ok();
        let mut file = ex::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // I mean, we just created it, so I don't expect it to fail
                    // cov:excl-start
                    let parent_dir = path.parent().unwrap_or(output_directory);
                    anyhow!("Output directory does not exist: {}", parent_dir.display())
                    // cov:excl-stop
                } else {
                    e.into()
                }
            })
            .with_context(|| {
                format!("Could not open completion marker file: {}", path.display())
            })?;
        file.write_all(b"run incomplete\n")?;
        file.sync_all()
            .with_context(|| format!("Failed to sync completion marker: {}", path.display()))?;
        Ok(OutputRunMarker { path, preexisting })
    }

    #[mutants::skip] // it's only precaution
    pub fn mark_complete(&self) -> Result<()> {
        match ex::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            // cov:excl-start
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err).with_context(|| {
                format!(
                    "Failed to remove completion marker after completion: {}",
                    self.path.display()
                )
            }),
            // cov:excl-stop
        }
    }

    pub fn was_preexisting(&self) -> bool {
        self.preexisting
    }
}
