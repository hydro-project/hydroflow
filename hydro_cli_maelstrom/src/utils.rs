use std::error::Error;
use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{stdin, AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

/// Spawns the specified child pricess with initialized pipes
pub fn spawn_child(
    path: &PathBuf,
) -> Result<(Child, ChildStdin, ChildStdout, ChildStderr), Box<dyn Error>> {
    let mut child = Command::new(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.take().ok_or("Could not access child's stdin")?;
    let child_stdout = child
        .stdout
        .take()
        .ok_or("Could not access child's stdin")?;
    let child_stderr = child
        .stderr
        .take()
        .ok_or("Could not access child's stdin")?;

    Ok((child, child_stdin, child_stdout, child_stderr))
}

/// Writes the specified message to the child's stdin
pub async fn write_to_child(
    child_stdin: &mut ChildStdin,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    let message = format!("{message}\n");
    let message_bytes = message.as_bytes();
    child_stdin.write_all(message_bytes).await?;
    #[cfg(debug_assertions)]
    println!("sent {}", message);

    Ok(())
}

/// Reads a line from the child's stdout
pub async fn read_from_child(child_stdout: &mut ChildStdout) -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();
    let mut reader = BufReader::new(child_stdout);
    reader.read_line(&mut buffer).await?;

    Ok(buffer)
}

/// Links two streams line-wise
pub async fn debug_link<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
    in_stream: R,
    mut out_stream: W,
    debug_message: String,
) {
    let mut lines = BufReader::new(in_stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let text = format!("{debug_message}: {line}\n");
        out_stream.write_all(text.as_bytes()).await.unwrap();
    }
}

/// Reads a line from stdin
pub async fn read_line() -> Result<String, Box<dyn Error>> {
    let mut buffer = String::new();
    BufReader::new(stdin()).read_line(&mut buffer).await?;

    Ok(buffer)
}
