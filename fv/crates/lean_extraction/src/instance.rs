use std::{
    fs,
    path::{Path, PathBuf},
};

use ff::Field;

use crate::{
    codegen::{FieldExporter, render_autogen_module},
    driver::ExtractionDriver,
    expr::Expr,
};

/// A trait for circuit instances that can be extracted by the driver.
pub trait CircuitInstance {
    type Field: Field + std::fmt::Debug + FieldExporter;

    /// Run the circuit on `dr` and return its output.
    /// The output is a vector of expressions corresponding to the
    /// output wires in order. This must include all "interesting" wires for which we
    /// want to prove some properties about.
    fn circuit(dr: &mut ExtractionDriver<Self::Field>)
    -> ragu_core::Result<Vec<Expr<Self::Field>>>;

    fn render_generated(module_name: &str) -> String {
        let mut dr = ExtractionDriver::<Self::Field>::new();
        let wires = Self::circuit(&mut dr).expect("circuit failed");
        render_autogen_module::<Self::Field>(module_name, dr.input_wire_count(), &dr.ops, &wires)
    }

    fn autogen_file_path(module_name: &str, autogen_root: impl AsRef<Path>) -> PathBuf {
        let mut path = autogen_root.as_ref().to_path_buf();
        for segment in module_name.split('.') {
            path.push(segment);
        }
        path.set_extension("lean");
        path
    }

    fn generated_file(module_name: &str, autogen_root: impl AsRef<Path>) -> (PathBuf, String) {
        (
            Self::autogen_file_path(module_name, autogen_root),
            Self::render_generated(module_name),
        )
    }

    /// Run the circuit and write the generated Lean module to the autogen tree.
    fn export(module_name: &str, autogen_root: impl AsRef<Path>) -> std::io::Result<PathBuf> {
        let (path, contents) = Self::generated_file(module_name, autogen_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents)?;
        Ok(path)
    }
}
