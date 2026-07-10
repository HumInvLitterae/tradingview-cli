use std::{fmt, io, process::ExitCode};

use serde::Serialize;
use tradingview_core::{AppError, ErrorBody, ErrorEnvelope, ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputDisposition {
    Written,
    BrokenPipe,
}

#[derive(Debug)]
pub(crate) enum OutputFailure {
    Serialization(serde_json::Error),
    Io(io::Error),
}

impl OutputFailure {
    pub(crate) fn into_app_error(self, destination: &str) -> AppError {
        AppError::new(
            ErrorKind::Internal,
            format!("Could not write JSON to {destination}: {self}"),
        )
    }
}

impl fmt::Display for OutputFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(error) => write!(formatter, "serialization failed: {error}"),
            Self::Io(error) => write!(formatter, "output failed: {error}"),
        }
    }
}

impl std::error::Error for OutputFailure {}

#[derive(Debug)]
pub(crate) enum JsonlRunError {
    Application(AppError),
    Stdout(OutputFailure),
    Stderr,
}

impl From<AppError> for JsonlRunError {
    fn from(error: AppError) -> Self {
        Self::Application(error)
    }
}

pub(crate) struct JsonlOutput<W> {
    writer: W,
}

impl<W> JsonlOutput<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: io::Write> JsonlOutput<W> {
    fn write<T: Serialize>(&mut self, value: &T) -> Result<OutputDisposition, OutputFailure> {
        write_json(&mut self.writer, value, JsonFormat::Compact)
    }
}

#[derive(Debug, Clone, Copy)]
enum JsonFormat {
    Pretty,
    Compact,
}

pub fn startup_error(message: impl Into<String>) -> ExitCode {
    let app_error = AppError::new(ErrorKind::Internal, message);
    let envelope = ErrorEnvelope::new("tv", ErrorBody::from(app_error));
    let _ = print_json_stderr(&envelope);
    ExitCode::from(1)
}

pub(crate) fn print_json_stdout<T: Serialize>(
    value: &T,
) -> Result<OutputDisposition, OutputFailure> {
    let stdout = io::stdout();
    write_json(&mut stdout.lock(), value, JsonFormat::Pretty)
}

pub(crate) fn print_json_stderr<T: Serialize>(
    value: &T,
) -> Result<OutputDisposition, OutputFailure> {
    let stderr = io::stderr();
    write_json(&mut stderr.lock(), value, JsonFormat::Pretty)
}

pub(crate) fn emit_jsonl_stdout<W, T>(
    output: &mut JsonlOutput<W>,
    value: &T,
) -> Result<OutputDisposition, JsonlRunError>
where
    W: io::Write,
    T: Serialize,
{
    output.write(value).map_err(JsonlRunError::Stdout)
}

pub(crate) fn emit_jsonl_stderr<W, T>(
    output: &mut JsonlOutput<W>,
    value: &T,
) -> Result<(), JsonlRunError>
where
    W: io::Write,
    T: Serialize,
{
    match output.write(value) {
        Ok(OutputDisposition::Written | OutputDisposition::BrokenPipe) => Ok(()),
        Err(_) => Err(JsonlRunError::Stderr),
    }
}

fn write_json<W, T>(
    writer: &mut W,
    value: &T,
    format: JsonFormat,
) -> Result<OutputDisposition, OutputFailure>
where
    W: io::Write,
    T: Serialize,
{
    let serialized = match format {
        JsonFormat::Pretty => serde_json::to_string_pretty(value),
        JsonFormat::Compact => serde_json::to_string(value),
    }
    .map_err(OutputFailure::Serialization)?;

    if write_bytes(writer, serialized.as_bytes())? == OutputDisposition::BrokenPipe {
        return Ok(OutputDisposition::BrokenPipe);
    }
    write_bytes(writer, b"\n")
}

fn write_bytes<W: io::Write>(
    writer: &mut W,
    bytes: &[u8],
) -> Result<OutputDisposition, OutputFailure> {
    match writer.write_all(bytes) {
        Ok(()) => Ok(OutputDisposition::Written),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
            Ok(OutputDisposition::BrokenPipe)
        }
        Err(error) => Err(OutputFailure::Io(error)),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use serde::ser::Error as _;
    use serde_json::json;

    use super::*;

    struct ErrorWriter {
        kind: io::ErrorKind,
    }

    impl Write for ErrorWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(self.kind, "test output failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(S::Error::custom("test serialization failure"))
        }
    }

    #[test]
    fn pretty_json_matches_existing_output_shape() {
        let mut output = Vec::new();
        let disposition = write_json(
            &mut output,
            &json!({"success": true, "data": {"count": 2}}),
            JsonFormat::Pretty,
        )
        .unwrap();

        assert_eq!(disposition, OutputDisposition::Written);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "{\n  \"data\": {\n    \"count\": 2\n  },\n  \"success\": true\n}\n"
        );
    }

    #[test]
    fn compact_jsonl_matches_existing_output_shape() {
        let mut output = Vec::new();
        let disposition = write_json(
            &mut output,
            &json!({"success": true, "data": {"count": 2}}),
            JsonFormat::Compact,
        )
        .unwrap();

        assert_eq!(disposition, OutputDisposition::Written);
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "{\"data\":{\"count\":2},\"success\":true}\n"
        );
    }

    #[test]
    fn broken_pipe_is_a_normal_disposition() {
        let mut writer = ErrorWriter {
            kind: io::ErrorKind::BrokenPipe,
        };

        let disposition =
            write_json(&mut writer, &json!({"ok": true}), JsonFormat::Compact).unwrap();

        assert_eq!(disposition, OutputDisposition::BrokenPipe);
    }

    #[test]
    fn other_io_failures_remain_failures() {
        let mut writer = ErrorWriter {
            kind: io::ErrorKind::PermissionDenied,
        };

        let error = write_json(&mut writer, &json!({"ok": true}), JsonFormat::Compact).unwrap_err();

        assert!(matches!(error, OutputFailure::Io(_)));
    }

    #[test]
    fn serialization_failures_remain_failures() {
        let error =
            write_json(&mut Vec::new(), &FailingSerialize, JsonFormat::Compact).unwrap_err();

        assert!(matches!(error, OutputFailure::Serialization(_)));
    }

    #[test]
    fn nonterminal_stderr_broken_pipe_is_suppressed() {
        let writer = ErrorWriter {
            kind: io::ErrorKind::BrokenPipe,
        };
        let mut output = JsonlOutput::new(writer);

        emit_jsonl_stderr(&mut output, &json!({"success": false})).unwrap();
    }

    #[test]
    fn nonterminal_stderr_other_failure_stops_the_runner() {
        let writer = ErrorWriter {
            kind: io::ErrorKind::PermissionDenied,
        };
        let mut output = JsonlOutput::new(writer);

        let error = emit_jsonl_stderr(&mut output, &json!({"success": false})).unwrap_err();

        assert!(matches!(error, JsonlRunError::Stderr));
    }
}
