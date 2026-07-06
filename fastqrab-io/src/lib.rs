use anyhow::{Context, bail};

pub mod blocks;
pub mod io;

pub use fastqrab_config::{CompressionFormat, FileFormat, STDIN_MAGIC_PATH, get_number_of_cores};

pub fn ensure_output_destination_available(
    path: &std::path::Path,
    allow_overwrite: bool,
    allow_fifo: bool,
) -> anyhow::Result<()> {
    use std::io::ErrorKind;

    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;

                if metadata.file_type().is_fifo() {
                    if allow_fifo {
                        return Ok(());
                    } else {
                        bail!(
                            "Output file \"{}\" was a fifo, but fifo not supported for this output",
                            path.display()
                        )
                    }
                }
            }

            if allow_overwrite {
                Ok(())
            } else {
                bail!(
                    "Output file \"{}\" already exists, refusing to overwrite. Pass --allow-overwrite to ignore this error.",
                    path.display(),
                );
            }
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            //mutants::skip
            // I mean that's basically expected.
            // missing directory is handled by the marker file creation
            Ok(())
        }
        // cov:excl-start
        Err(err) => {
            Err(err).with_context(|| format!("Could not inspect existing path: {}", path.display()))
        } // cov:excl-stop
    }
}
