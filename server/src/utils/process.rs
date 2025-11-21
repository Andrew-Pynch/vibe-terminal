use std::process::Command;
use std::io;

/// Spawns a child process and captures its stdout and stderr.
///
/// # Arguments
/// * `command` - The command to execute (e.g., "ls", "echo").
/// * `args` - A vector of string slices representing the arguments for the command.
///
/// # Returns
/// A `Result` containing a tuple of (stdout, stderr) strings on success,
/// or an `io::Error` if the process fails to spawn or execute.
///
/// # Errors
/// This function will return an `io::Error` if:
/// - The command cannot be spawned (e.g., command not found).
/// - The command exits with a non-zero status code (indicating an error in the executed program itself).
pub fn spawn_and_capture_output(
    command: &str,
    args: &[&str],
) -> io::Result<(String, String)> {
    let output = Command::new(command)
        .args(args)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        // Return an error if the command itself failed (non-zero exit code)
        let error_message = format!(
            "Command '{} {}' failed with exit code {:?}\nStdout: {}\nStderr: {}",
            command,
            args.join(" "),
            output.status.code(),
            stdout,
            stderr
        );
        return Err(io::Error::new(io::ErrorKind::Other, error_message));
    }

    Ok((stdout, stderr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn test_spawn_and_capture_echo() {
        let (stdout, stderr) = 
            spawn_and_capture_output("echo", &["hello world"]).expect("Failed to run echo command");
        assert_eq!(stdout.trim(), "hello world");
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_spawn_and_capture_ls_no_args() {
        let (stdout, stderr) = 
            spawn_and_capture_output("ls", &[]).expect("Failed to run ls command");
        // We can't assert specific output for 'ls' reliably as it depends on the environment
        // but we can assert it produced some output and no stderr for a successful run.
        assert!(!stdout.is_empty());
        assert!(stderr.is_empty());
    }

    #[test]
    fn test_spawn_and_capture_command_with_stderr() {
        // This command will write to stderr
        let (stdout, stderr) = spawn_and_capture_output(
            "bash",
            &["-c", "echo 'this is an error' >&2; echo 'this is stdout'"],
        )
        .expect("Failed to run bash command with stderr");
        assert_eq!(stdout.trim(), "this is stdout");
        assert_eq!(stderr.trim(), "this is an error");
    }

    #[test]
    fn test_spawn_and_capture_non_existent_command() {
        let result = spawn_and_capture_output("non_existent_command_123", &[]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn test_spawn_and_capture_command_with_non_zero_exit_code() {
        let result = spawn_and_capture_output("bash", &["-c", "exit 1"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert!(err.to_string().contains("failed with exit code 1"));
    }

    #[test]
    fn test_spawn_and_capture_with_input_file() {
        let dir = tempdir().expect("Failed to create temporary directory");
        let file_path = dir.path().join("test_input.txt");
        fs::write(&file_path, "line 1\nline 2").expect("Failed to write to test_input.txt");

        // Use 'cat' to read the file
        let (stdout, stderr) = spawn_and_capture_output(
            "cat",
            &[file_path.to_str().unwrap()],
        )
        .expect("Failed to run cat command");

        assert_eq!(stdout.trim(), "line 1\nline 2");
        assert!(stderr.is_empty());
    }
}
