use std::path::{Path, PathBuf};
use thiserror::Error;

/// A code builder. To detect error code.
pub enum CodeBuilder<'a> {
    Path(&'a Path),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a build error returned from running cargo build.
pub struct BuildError<'a> {
    pub error_code: Option<String>,
    pub source_file: Option<PathBuf>,
    pub error_src: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildErros<'a> {
    pub errors: Vec<BuildError<'a>>,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unmatched location information")]
    UnmatchedLocationInformation,
    #[error("Invalid string found, {0}")]
    InvalidString(String),
}

/*
 *
error[E0384]: cannot assign twice to immutable variable `a`
 --> test/test_project/src/main.rs:4:5
error: could not compile `test_project` (bin "test_project") due to previous error; 3 warnings emitted
 * */

impl<'a> TryFrom<&'a str> for BuildErros<'a> {
    type Error = ParseError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
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
                    error_src: line,
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

impl CodeBuilder<'_> {
    pub fn collect_errors(self) {
        match self {
            CodeBuilder::Path(src_code_path) => {
                // TODO: IMPLEMENT THIS
                // 1.Check if source code path is a cargo project.
                // 2.Run cargo build in the src_code_path.
                // 3.Collect all error codes.
                unimplemented!("code builder with path is not implemented yet.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildError, BuildErros};

    #[test]
    fn test_parse_single_error_code() {
        let test_cargo_output = r#"
error[E0384]: cannot assign twice to immutable variable `a`
 --> test/test_project/src/main.rs:4:5
error: could not compile `test_project` (bin "test_project") due to previous error; 3 warnings emitted
"#;

        let build_errors = BuildErros::try_from(test_cargo_output).unwrap();

        let expected_error = BuildError {
            error_code: Some("E0384".to_owned()),
            source_file: Some("test/test_project/src/main.rs".into()),
            error_src: "error[E0384]: cannot assign twice to immutable variable `a`",
        };

        let expected_build_errors = BuildErros {
            errors: vec![expected_error],
        };

        assert_eq!(expected_build_errors, build_errors);
    }
}
