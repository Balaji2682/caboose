use portable_pty::{ChildKiller, CommandBuilder, PtySize, native_pty_system};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

enum ChildHandle {
    Pty {
        killer: Box<dyn ChildKiller + Send + Sync>,
        child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    },
    Plain {
        child: Arc<Mutex<std::process::Child>>,
    },
}

impl Clone for ChildHandle {
    fn clone(&self) -> Self {
        match self {
            ChildHandle::Pty { killer, child } => ChildHandle::Pty {
                killer: killer.clone_killer(),
                child: child.clone(),
            },
            ChildHandle::Plain { child } => ChildHandle::Plain {
                child: child.clone(),
            },
        }
    }
}

impl ChildHandle {
    fn kill(&self) -> Result<(), String> {
        match self {
            ChildHandle::Pty { killer, child } => {
                // Clone the killer to get a mutable instance for the kill operation
                let mut mutable_killer = killer.clone_killer();
                mutable_killer
                    .kill()
                    .map_err(|e| format!("Failed to kill PTY child: {}", e))?;
                // Best-effort reap without holding the lock long
                let _ = child
                    .lock()
                    .map_err(|_| "Failed to lock PTY child".to_string())?
                    .try_wait();
                Ok(())
            }
            ChildHandle::Plain { child } => {
                let mut child = child
                    .lock()
                    .map_err(|_| "Failed to lock process".to_string())?;
                // Ignore errors from killing an already exited process
                let _ = child.kill();
                let _ = child.wait();
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Stopped,
    Crashed,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub command: String,
    pub status: ProcessStatus,
    pub start_time: Option<Instant>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub process_name: String,
    pub content: String,
    pub timestamp: Instant,
}

pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
    child_handles: Arc<Mutex<HashMap<String, ChildHandle>>>,
    log_tx: mpsc::UnboundedSender<LogLine>,
    use_pty: bool,
}

impl ProcessManager {
    pub fn new(log_tx: mpsc::UnboundedSender<LogLine>) -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            child_handles: Arc::new(Mutex::new(HashMap::new())),
            log_tx,
            use_pty: std::env::var("NO_PTY").is_err(),
        }
    }

    pub fn spawn_process(
        &self,
        name: String,
        command: String,
        env_vars: HashMap<String, String>,
    ) -> Result<(), String> {
        // Pre-register process so UI shows it even if spawn fails
        {
            let mut processes = self.processes.lock().unwrap();
            processes.insert(
                name.clone(),
                ProcessInfo {
                    name: name.clone(),
                    command: command.clone(),
                    status: ProcessStatus::Running,
                    start_time: Some(Instant::now()),
                    pid: None,
                },
            );
        }

        if self.use_pty {
            self.spawn_with_pty(name, command, env_vars)
        } else {
            self.spawn_without_pty(name, command, env_vars)
        }
    }

    fn spawn_with_pty(
        &self,
        name: String,
        command: String,
        env_vars: HashMap<String, String>,
    ) -> Result<(), String> {
        let pty_system = native_pty_system();

        let (program, args) = parse_command(&command)?;

        let mut cmd = CommandBuilder::new(&program);
        for arg in args {
            cmd.arg(arg);
        }

        // Set working directory to current directory
        if let Ok(current_dir) = std::env::current_dir() {
            cmd.cwd(current_dir);
        }

        // Add environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        // Create PTY pair
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        // Spawn the process
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let pid = child.process_id();
        let killer = child.clone_killer();
        let child = Arc::new(Mutex::new(child));

        // Update process info
        {
            let mut processes = self.processes.lock().unwrap();
            if let Some(info) = processes.get_mut(&name) {
                info.pid = pid;
            }
        }

        // Track child handle for cleanup
        {
            let mut handles = self.child_handles.lock().unwrap();
            handles.insert(
                name.clone(),
                ChildHandle::Pty {
                    killer,
                    child: child.clone(),
                },
            );
        }

        // Read from PTY and send to log channel
        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;

        let log_tx = self.log_tx.clone();
        let process_name = name.clone();
        let processes = self.processes.clone();

        tokio::spawn(async move {
            let buf_reader = BufReader::new(reader);
            for line in buf_reader.lines() {
                match line {
                    Ok(content) => {
                        // Strip ANSI escape codes (colors, cursor movement, spinners, etc.)
                        // to prevent them from bleeding into the TUI
                        let bytes = strip_ansi_escapes::strip(&content);
                        let cleaned_content = String::from_utf8_lossy(&bytes).to_string();

                        let _ = log_tx.send(LogLine {
                            process_name: process_name.clone(),
                            content: cleaned_content,
                            timestamp: Instant::now(),
                        });
                    }
                    Err(_) => break,
                }
            }

            // Process ended
            let mut procs = processes.lock().unwrap();
            if let Some(info) = procs.get_mut(&process_name) {
                info.status = ProcessStatus::Stopped;
            }
        });

        // Monitor child process
        let process_name = name.clone();
        let processes = self.processes.clone();
        let child_handles = self.child_handles.clone();
        let child_for_monitor = child.clone();
        tokio::spawn(async move {
            loop {
                let done = {
                    let mut guard = child_for_monitor.lock().unwrap();
                    match guard.try_wait() {
                        Ok(Some(_)) => true,
                        Ok(None) => false,
                        Err(_) => true,
                    }
                };
                if done {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }

            let mut procs = processes.lock().unwrap();
            if let Some(info) = procs.get_mut(&process_name) {
                info.status = ProcessStatus::Stopped;
            }
            let mut handles = child_handles.lock().unwrap();
            handles.remove(&process_name);
        });

        Ok(())
    }

    fn spawn_without_pty(
        &self,
        name: String,
        command: String,
        env_vars: HashMap<String, String>,
    ) -> Result<(), String> {
        let (program, args) = parse_command(&command)?;

        let mut cmd = std::process::Command::new(&program);
        cmd.args(&args);

        // Set working directory to current directory
        if let Ok(current_dir) = std::env::current_dir() {
            cmd.current_dir(current_dir);
        }

        cmd.envs(env_vars);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;
        let pid = child.id();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let child = Arc::new(Mutex::new(child));

        {
            let mut processes = self.processes.lock().unwrap();
            if let Some(info) = processes.get_mut(&name) {
                info.pid = Some(pid);
            }
        }

        // Track child handle for cleanup
        {
            let mut handles = self.child_handles.lock().unwrap();
            handles.insert(
                name.clone(),
                ChildHandle::Plain {
                    child: child.clone(),
                },
            );
        }

        // stdout
        if let Some(stdout) = stdout {
            let log_tx = self.log_tx.clone();
            let process_name = name.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(content) = line {
                        // Strip ANSI escape codes to prevent TUI bleeding
                        let bytes = strip_ansi_escapes::strip(&content);
                        let cleaned_content = String::from_utf8_lossy(&bytes).to_string();

                        let _ = log_tx.send(LogLine {
                            process_name: process_name.clone(),
                            content: cleaned_content,
                            timestamp: Instant::now(),
                        });
                    }
                }
            });
        }

        // stderr
        if let Some(stderr) = stderr {
            let log_tx = self.log_tx.clone();
            let process_name = name.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(content) = line {
                        // Strip ANSI escape codes to prevent TUI bleeding
                        let bytes = strip_ansi_escapes::strip(&content);
                        let cleaned_content = String::from_utf8_lossy(&bytes).to_string();

                        let _ = log_tx.send(LogLine {
                            process_name: process_name.clone(),
                            content: cleaned_content,
                            timestamp: Instant::now(),
                        });
                    }
                }
            });
        }

        // Monitor child process
        let processes = self.processes.clone();
        let process_name = name.clone();
        let child_handles = self.child_handles.clone();
        let child = child.clone();
        tokio::spawn(async move {
            loop {
                let done = {
                    let mut guard = child.lock().unwrap();
                    match guard.try_wait() {
                        Ok(Some(_)) => true,
                        Ok(None) => false,
                        Err(_) => true,
                    }
                };
                if done {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }
            let mut procs = processes.lock().unwrap();
            if let Some(info) = procs.get_mut(&process_name) {
                info.status = ProcessStatus::Stopped;
            }
            let mut handles = child_handles.lock().unwrap();
            handles.remove(&process_name);
        });

        Ok(())
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        let processes = self.processes.lock().unwrap();
        processes.values().cloned().collect()
    }

    pub fn get_process(&self, name: &str) -> Option<ProcessInfo> {
        let processes = self.processes.lock().unwrap();
        processes.get(name).cloned()
    }

    pub fn stop_all(&self) {
        let handles: Vec<(String, ChildHandle)> = {
            let handles = self.child_handles.lock().unwrap();
            handles
                .iter()
                .map(|(name, handle)| (name.clone(), handle.clone()))
                .collect()
        };

        for (name, handle) in handles {
            if let Err(err) = handle.kill() {
                eprintln!("Failed to stop process {}: {}", name, err);
            }
        }

        {
            let mut processes = self.processes.lock().unwrap();
            for info in processes.values_mut() {
                info.status = ProcessStatus::Stopped;
            }
        }

        let mut handles = self.child_handles.lock().unwrap();
        handles.clear();
    }
}

fn parse_command(command: &str) -> Result<(String, Vec<String>), String> {
    if command.trim().is_empty() {
        return Err("Empty command".to_string());
    }

    if should_use_shell(command) {
        let shell = preferred_shell();
        return Ok((
            shell.to_string(),
            vec!["-lc".to_string(), command.to_string()],
        ));
    }

    let parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();

    if parts.is_empty() {
        return Err("Empty command".to_string());
    }

    let program = parts[0].clone();
    let args = parts[1..].to_vec();
    Ok((program, args))
}

fn should_use_shell(command: &str) -> bool {
    command.contains("&&")
        || command.contains("||")
        || command.contains('|')
        || command.contains(';')
        || command.contains("cd ")
}

fn preferred_shell() -> &'static str {
    // Use bash for better compatibility (e.g., scripts that use [[ ]])
    if PathBuf::from("/usr/bin/bash").exists() {
        "bash"
    } else {
        "sh"
    }
}
