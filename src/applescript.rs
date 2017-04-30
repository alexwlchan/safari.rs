use std::process::{Command, ExitStatus};


/// The output of a finished process.
///
/// This varies from Output in std::process in that stdout/stderr are
/// both strings rather than Vec<u8>.
pub struct Output {
  pub status: ExitStatus,
  pub stdout: String,
  pub stderr: String,
}


/// Run an AppleScript.
///
/// * `script`: The AppleScript code to run.
///
pub fn run(script: &str) -> Output {
  let cmd_result = Command::new("osascript")
    .arg("-e")
    .arg(script)
    .output()
    .expect("failed to execute AppleScript");

  Output {
    status: cmd_result.status,
    stdout: String::from_utf8(cmd_result.stdout).unwrap(),
    stderr: String::from_utf8(cmd_result.stderr).unwrap(),
  }
}
