use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use thiserror::Error;

/// A code builder. To detect error code.
pub enum CodeBuilder<'a> {
    Path(&'a Path),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a build error returned from running cargo build.
pub struct BuildError {
    pub error_code: Option<String>,
    pub source_file: Option<PathBuf>,
    pub error_src: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildErros {
    pub errors: Vec<BuildError>,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unmatched location information")]
    UnmatchedLocationInformation,
}

#[derive(Error, Debug)]
pub enum CodeBuilderError {
    #[error("IO eror emitted from code builder: {0}")]
    IOError(std::io::Error),
    #[error("Cargo output parse error: {0}")]
    CargoOutputParseError(ParseError),
}

impl From<std::io::Error> for CodeBuilderError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<ParseError> for CodeBuilderError {
    fn from(value: ParseError) -> Self {
        Self::CargoOutputParseError(value)
    }
}

impl TryFrom<String> for BuildErros {
    type Error = ParseError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut current_error = None;
        let mut errors = vec![];
        for line in value.trim().lines() {
            let line = line.trim();
            if line.starts_with("error") {
                // We found an error line.

                // Check if we have an error code.
                let error_code = line
                    .split('[')
                    .nth(1)
                    .and_then(|line| line.split(']').next())
                    .map(|code| code.to_string());
                current_error = Some(BuildError {
                    error_code,
                    source_file: None,
                    error_src: line.to_string(),
                });
            } else if line.trim().starts_with("-->") {
                // We found location information for the current error.

                // We should have a currently active error if not this is not a valid output for
                // our tool.
                let mut error = current_error
                    .clone()
                    .ok_or(ParseError::UnmatchedLocationInformation)?;

                let loc_info = line.split('>').nth(1).map(|loc_info| loc_info.trim());
                let path = loc_info
                    .and_then(|loc_info| loc_info.split(':').next())
                    .map(|loc_info| loc_info.into());
                error.source_file = path;

                errors.push(error);
                current_error = None;
            }
        }
        Ok(Self { errors })
    }
}

impl<'a> CodeBuilder<'a> {
    pub fn collect_errors(&'a self) -> Result<BuildErros, CodeBuilderError> {
        match self {
            CodeBuilder::Path(src_code_path) => {
                let build_output = execute_cargo_check_and_grep(src_code_path)?;
                dbg!(build_output.clone());
                //let build_output =  String::from_utf8(build_output.stderr)?;
                // TODO: IMPLEMENT THIS
                // 1.Check if source code path is a cargo project.
                // 2.Run cargo build in the src_code_path.
                // 3.Collect all error codes.
                Ok(BuildErros::try_from(build_output)?)
            }
        }
    }
}

fn execute_cargo_check_and_grep(path: &Path) -> Result<String, std::io::Error> {
    // Run `cargo build` and capture its output
    let cargo_output = Command::new("cargo")
        .current_dir(path)
        .arg("build")
        .stderr(Stdio::piped())
        .output()?;

    // Prepare `ripgrep` command with the desired pattern
    let grep_output = Command::new("rg")
        .current_dir(path)
        .arg("-i")
        .arg("--multiline")
        .arg("(^error.*\\n.*)|(aborting)")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write cargo's output to `ripgrep`'s stdin
    let mut grep_stdin = grep_output
        .stdin
        .as_ref()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to open rg stdin"))?;
    grep_stdin.write_all(&cargo_output.stderr)?;

    // Collect the output from `ripgrep`
    let grep_result = grep_output.wait_with_output()?;

    // Convert the output to a String and return it
    String::from_utf8(grep_result.stdout)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{BuildError, BuildErros, CodeBuilder};

    #[test]
    fn test_parse_single_error_code() {
        let test_cargo_output = r#"
error[E0384]: cannot assign twice to immutable variable `a`
 --> test/test_project/src/main.rs:4:5
error: could not compile `test_project` (bin "test_project") due to previous error; 3 warnings emitted
"#;

        let build_errors = BuildErros::try_from(test_cargo_output.to_string()).unwrap();

        let expected_error = BuildError {
            error_code: Some("E0384".to_owned()),
            source_file: Some("test/test_project/src/main.rs".into()),
            error_src: "error[E0384]: cannot assign twice to immutable variable `a`".to_owned(),
        };

        let expected_build_errors = BuildErros {
            errors: vec![expected_error],
        };

        assert_eq!(expected_build_errors, build_errors);
    }

    #[test]
    fn test_collect_errors_test_project() {
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test")
            .join("data")
            .join("test_project");
        let code_builder = CodeBuilder::Path(&project_dir);

        let errors = code_builder.collect_errors().unwrap();

        let expected_error = BuildError {
            error_code: Some("E0384".to_owned()),
            source_file: Some("src/main.rs".into()),
            error_src: "error[E0384]: cannot assign twice to immutable variable `a`".to_owned(),
        };

        let expected_build_errors = BuildErros {
            errors: vec![expected_error],
        };

        assert_eq!(errors, expected_build_errors)
    }
}
