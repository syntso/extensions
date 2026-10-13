use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("cannot read {}: {source}", path.display())]
    Read { path: PathBuf, source: io::Error },

    #[error("cannot write {}: {source}", path.display())]
    Write { path: PathBuf, source: io::Error },

    #[error("{} is not valid: {source}", path.display())]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("{}", list(.0))]
    Invalid(Vec<String>),

    #[error("cannot start `{program}`: {source}")]
    Spawn { program: String, source: io::Error },

    #[error("`{program}` failed with {status}")]
    Failed { program: String, status: ExitStatus },

    #[error("cargo built no WebAssembly component for {0}")]
    NoComponent(String),

    #[error("{0} is not set")]
    MissingVariable(&'static str),

    #[error("minisign: {0}")]
    Signature(#[from] minisign::PError),

    #[error("cannot write the index: {0}")]
    Index(#[from] serde_json::Error),
}

fn list(problems: &[String]) -> String {
    let count = problems.len();
    let lines: Vec<String> = problems
        .iter()
        .map(|problem| format!("\n  {problem}"))
        .collect();

    format!("{count} problem(s):{}", lines.concat())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_every_problem() {
        let error = RegistryError::Invalid(vec!["a: one".to_owned(), "b: two".to_owned()]);

        assert_eq!(error.to_string(), "2 problem(s):\n  a: one\n  b: two");
    }
}
