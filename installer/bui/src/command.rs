use std::process::Command;
use std::path::Path;
use std;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RunningCommand {
    pub command: Vec<String>,
    pub output: String,
    pub error: String,
    pub exit_code: Option<i32>,
}

pub fn rm_f(file: &str) -> Result<(), std::io::Error> {
    if Path::new(file).exists() {
        std::fs::remove_file(file)?;
    }
    Ok(())
}

pub fn reset_background() -> Result<(), std::io::Error> {
    rm_f("cmd.txt")?;
    rm_f("exit_code.txt")?;
    rm_f("stderr.log")?;
    rm_f("out.log")?;
    Ok(())
}

pub enum ExitCodeError {
  StorageError(String),
  IOError(std::io::Error),
}
impl From<std::io::Error> for ExitCodeError {
    fn from(err: std::io::Error) -> ExitCodeError { ExitCodeError::IOError(err) }
}

pub fn background_exit_code() -> Result<i32, ExitCodeError> {
    if !Path::new("exit_code.txt").exists() {
        let msg = "exit_code.txt does not exist";
        return Err(ExitCodeError::StorageError(msg.to_string()));
    }
    let exit_code = tail("exit_code.txt")?;
    match exit_code.parse::<i32>() {
        Ok(code) => Ok(code)
      , Err(err) =>
          Err(ExitCodeError::StorageError(format!("{:}", err)))
    }
}

pub fn background(command: String) -> Result<RunningCommand, std::io::Error> {
  let just_recipe = command.clone();
  reset_background()?;

  std::thread::spawn(|| {
    let log_command = command.clone();
    let mut process = Command::new("runner.sh")
            .arg("just")
            .arg(command)
            .stdout(std::process::Stdio::piped())
            .spawn().ok().expect("just command failed to run");
    match process.wait() {
        Ok(_) => { ; }
        Err(err) => error!("{}", err)
    }

    println!("command done: {}", log_command);
  });

  Ok(default_running_command(vec!["just".to_string(), just_recipe]))
}

fn default_running_command(command: Vec<String>) -> RunningCommand {
    RunningCommand {
        command: command,
        output: "".into(),
        error: "".into(),
        exit_code: None,
    }

}

pub fn update_running_command() -> Result<Option<RunningCommand>, std::io::Error> {
    if !Path::new("cmd.txt").exists() {
        return Ok(None);
    }

    let mut running = default_running_command(tail("cmd.txt")?
                        .split(" ").map(|str| str.to_string()).collect());
    if Path::new("out.log").exists() {
      running.output = tail("out.log")?;
      match background_exit_code() {
          Err(_) => {}
       ,  Ok(code) => {
            running.exit_code = Some(code);
            if code != 0 {
              let stderr = tail("stderr.log")?;
              if stderr.is_empty() {
                running.error = "error running command".to_string();
              }
            }
          }
      }
    }
    Ok(Some(running))
}

pub fn tail(file: &str) -> Result<String, std::io::Error> {
    let output = Command::new("tail")
            .arg("-1000")
            .arg(file)
            .stdout(std::process::Stdio::piped())
            .spawn()?.wait_with_output()?;
    Ok(String::from_utf8_lossy(output.stdout.as_slice()).to_string())
}

pub fn mysql_connect() -> Result<String, std::io::Error> {
    let output = Command::new("just")
        .arg("mysql-connect")
        .stdout(std::process::Stdio::piped())
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
