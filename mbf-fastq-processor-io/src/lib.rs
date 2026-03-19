pub mod io;

use anyhow::Context;

pub use mbf_fastq_processor_config::{
    CompressionFormat, FileFormat, STDIN_MAGIC_PATH, get_number_of_cores,
};

pub fn ensure_output_destination_available(
    path: &std::path::Path,
    allow_overwrite: bool,
) -> anyhow::Result<Option<std::fs::Metadata>> {
    use std::io::ErrorKind;

    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;

                if metadata.file_type().is_fifo() {
                    return Ok(Some(metadata));
                }
            }

            if allow_overwrite {
                return Ok(Some(metadata));
            }

            anyhow::bail!(
                "Output file \"{}\" already exists, refusing to overwrite. Pass --allow-overwrite to ignore this error.",
                path.display(),
            );
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            // I mean that's basically expected.
            // missing directory is handled by the marker file creation
            Ok(None)
        }
        // cov:excl-start
        Err(err) => {
            Err(err).with_context(|| format!("Could not inspect existing path: {}", path.display()))
        } // cov:excl-stop
    }
}
