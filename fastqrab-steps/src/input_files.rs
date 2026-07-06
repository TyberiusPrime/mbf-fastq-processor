//! Auxiliary input files declared by steps.
//!
//! Some steps read files besides the pipeline `[input]` reads (e.g.
//! `HammingCorrect` reading a counts report). Rather than each step opening
//! arbitrary paths itself at run time, a step *declares* the paths it wants via
//! [`TagUser::declare_input_files`](crate::transformations::TagUser::declare_input_files)
//! at config-verification time; the runtime opens them once and hands the
//! handles to [`Step::init`](crate::transformations::Step::init) through
//! [`StepInputFiles`]. This mirrors the `OutputDeclaration` / `StepOutputFiles`
//! machinery on the output side, and gives a single place that knows the full
//! set of out-of-tree reads ahead of processing — the hook a future
//! filesystem sandbox will build its read-allowlist from.

use std::collections::HashMap;
use std::path::PathBuf;

/// One auxiliary input file a step wants to read.
///
/// Returned from [`TagUser::declare_input_files`](crate::transformations::TagUser::declare_input_files).
/// The runtime opens `path` for reading and hands the handle to `init` keyed by
/// `id`, which the step uses to retrieve it from [`StepInputFiles`].
#[derive(Debug, Clone)]
pub struct InputDeclaration {
    /// Step-local key used to retrieve the opened handle from [`StepInputFiles`].
    pub id: String,
    /// Path to open for reading.
    pub path: PathBuf,
}

/// Per-step input handles handed to [`Step::init`](crate::transformations::Step::init).
/// Keyed by the `id` from each [`InputDeclaration`].
pub struct StepInputFiles(pub HashMap<String, ex::fs::File>);

impl std::fmt::Debug for StepInputFiles {
    //cov:excl-start
    #[mutants::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `ex::fs::File` is not `Debug`; the ids are the useful part anyway.
        f.debug_struct("StepInputFiles")
            .field("ids", &self.0.keys().collect::<Vec<_>>())
            .finish()
    }
    //cov::excl-stop
}

impl StepInputFiles {
    #[must_use]
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    /// Take the opened handle for a declared input id. Panics if the id is
    /// unknown — it must match one returned from `declare_input_files`.
    /// # Panics
    /// on an unknown id.
    #[must_use]
    pub fn take(&mut self, id: &str) -> ex::fs::File {
        self.0
            .remove(id)
            .unwrap_or_else(|| panic!("StepInputFiles: unknown input id '{id}'"))
    }

    /// Assert every declared input handle was taken during `init`. A leftover
    /// means a step declared a file via `declare_input_files` but never
    /// `take`-d it — a step bug we want to surface loudly.
    /// # Panics
    /// if any declared input handle was not consumed.
    pub fn assert_consumed(&self) {
        assert!(
            self.0.is_empty(),
            "StepInputFiles: declared input(s) not consumed in init: {:?}",
            self.0.keys().collect::<Vec<_>>() // cov:excl-line
        );
    }
}
