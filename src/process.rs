use std::process::{Command, Stdio};

use crate::error::RegistryError;

pub fn run(command: &mut Command) -> Result<String, RegistryError> {
    let program = describe(command);

    let output = command
        .stderr(Stdio::inherit())
        .output()
        .map_err(|source| RegistryError::Spawn {
            program: program.clone(),
            source,
        })?;

    if !output.status.success() {
        return Err(RegistryError::Failed {
            program,
            status: output.status,
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn describe(command: &Command) -> String {
    let arguments: Vec<String> = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();

    format!(
        "{} {}",
        command.get_program().to_string_lossy(),
        arguments.join(" ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_stdout() {
        let stdout = run(Command::new("cargo").arg("--version")).expect("cargo runs");

        assert!(stdout.starts_with("cargo "));
    }

    #[test]
    fn fails_on_a_bad_status() {
        let error = run(Command::new("cargo").arg("no-such-subcommand")).expect_err("fails");

        assert!(matches!(error, RegistryError::Failed { .. }));
    }

    #[test]
    fn fails_on_a_missing_program() {
        let error = run(&mut Command::new("no-such-program-here")).expect_err("missing");

        assert!(matches!(error, RegistryError::Spawn { .. }));
    }
}
