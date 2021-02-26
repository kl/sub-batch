use crate::config::{GlobalConfig, TimeConfig};
use crate::scanner;
use crate::scanner::{ScanOptions, SubAndFile};
use crate::time;
use anyhow::Context;
use anyhow::Result as AnyResult;
use crossterm::cursor::MoveToColumn;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, event, terminal, ExecutableCommand};
use interprocess::local_socket::LocalSocketStream;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

pub struct MpvCommand {
    global_conf: GlobalConfig,
}

impl MpvCommand {
    const SHIFT_LARGE: i64 = 500;
    const SHIFT_MEDIUM: i64 = 250;
    const SHIFT_SMALL: i64 = 50;

    pub fn new(global_conf: GlobalConfig) -> Self {
        Self { global_conf }
    }

    pub fn run(&self) -> AnyResult<()> {
        let mpv = which::which("mpv").context("could not find `mpv` in PATH. Is mpv installed?")?;
        let target = self.first_sub_video_match()?;
        let socket_file = mpv_socket_file()?;

        let mut child = Command::new(&mpv)
            .arg(&target.file_path)
            .arg(format!("--input-ipc-server={}", socket_file.display()))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        let mut conn = MpvConnection::connect(&socket_file)?;

        self.start_shift_loop(&mut conn)?;
        println!();

        let _ = child.kill();
        let _ = child.wait();
        Ok(())
    }

    fn first_sub_video_match(&self) -> AnyResult<SubAndFile> {
        let mut matches = scanner::scan(ScanOptions::path_only(&self.global_conf.path))?;
        if matches.is_empty() {
            bail!("must have at least one matching video/subtitle file pair");
        }
        Ok(matches.swap_remove(0))
    }

    fn start_shift_loop(&self, conn: &mut MpvConnection) -> AnyResult<()> {
        terminal::enable_raw_mode()?;
        io::stdout().execute(cursor::Hide)?;
        self.print_banner()?;

        let mut time_shift: i64 = 0;

        loop {
            let event = event::read()?;
            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }

            if let Event::Key(KeyEvent {
                code: KeyCode::Char(char),
                modifiers,
            }) = event
            {
                time_shift += match char {
                    '1' => self.shift_subs(conn, -Self::SHIFT_LARGE)?,
                    '2' => self.shift_subs(conn, -Self::SHIFT_MEDIUM)?,
                    '3' => self.shift_subs(conn, -Self::SHIFT_SMALL)?,
                    '4' => self.shift_subs(conn, Self::SHIFT_SMALL)?,
                    '5' => self.shift_subs(conn, Self::SHIFT_MEDIUM)?,
                    '6' => self.shift_subs(conn, Self::SHIFT_LARGE)?,
                    'c' if modifiers == KeyModifiers::CONTROL => break,
                    _ => 0,
                };

                io::stdout()
                    .execute(Clear(ClearType::CurrentLine))?
                    .execute(MoveToColumn(0))?
                    .execute(Print(format!("shift: {}ms", time_shift)))?;
            }
        }

        io::stdout().execute(cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn print_banner(&self) -> AnyResult<()> {
        io::stdout()
            .execute(Print(format!("LEFT  {}ms\t1\n", Self::SHIFT_LARGE)))?
            .execute(MoveToColumn(0))?
            .execute(Print(format!("LEFT  {}ms\t2\n", Self::SHIFT_MEDIUM)))?
            .execute(MoveToColumn(0))?
            .execute(Print(format!("LEFT  {}ms\t3\n", Self::SHIFT_SMALL)))?
            .execute(MoveToColumn(0))?
            .execute(Print(format!("RIGHT {}ms\t4\n", Self::SHIFT_SMALL)))?
            .execute(MoveToColumn(0))?
            .execute(Print(format!("RIGHT {}ms\t5\n", Self::SHIFT_MEDIUM)))?
            .execute(MoveToColumn(0))?
            .execute(Print(format!("RIGHT {}ms\t6\n", Self::SHIFT_LARGE)))?
            .execute(MoveToColumn(0))?
            .execute(Print("QUIT\t\t<ESC>\n\n"))?
            .execute(MoveToColumn(0))?
            .execute(Print("shift: 0ms"))?;
        Ok(())
    }

    fn shift_subs(&self, conn: &mut MpvConnection, timing: i64) -> AnyResult<i64> {
        time::run(&self.global_conf, TimeConfig::timing(timing))?;
        let resp = conn.send_wait(r#"{ "command": ["sub_reload"] }"#)?;
        if !resp.contains("success") {
            bail!(resp);
        }
        Ok(timing)
    }
}

fn mpv_socket_file() -> AnyResult<PathBuf> {
    Ok(tempfile::NamedTempFile::new()?.path().into())
}

struct MpvConnection {
    stream: BufReader<LocalSocketStream>,
}

impl MpvConnection {
    const RETRY_INTERVAL: Duration = Duration::from_millis(200);
    const RETRY_ATTEMPTS: i32 = 10;

    fn connect(local_socket: &Path) -> io::Result<Self> {
        let mut tries = 0;
        loop {
            match LocalSocketStream::connect(local_socket) {
                Ok(stream) => {
                    break Ok(Self {
                        stream: BufReader::new(stream),
                    })
                }
                Err(err) => {
                    tries += 1;
                    if tries == Self::RETRY_ATTEMPTS {
                        break Err(err);
                    }
                    thread::sleep(Self::RETRY_INTERVAL);
                }
            }
        }
    }

    /// Send the command (`cmd` should not have a newline in it) and wait for and return
    /// the response from mpv.
    fn send_wait(&mut self, cmd: &str) -> AnyResult<String> {
        let req_id = "45782199";
        let close = cmd.rfind('}').expect("invalid send command");
        let open = &cmd[0..close];
        let new = format!(r#"{}, "request_id": {} {}"#, open, req_id, "}");
        self.send(&new)?;

        let mut buf = String::new();
        loop {
            self.stream.read_line(&mut buf)?;
            if buf.contains(req_id) {
                return Ok(buf);
            }
            buf.clear();
        }
    }

    /// Send the command (`cmd` should not have a newline in it).
    fn send(&mut self, cmd: &str) -> io::Result<()> {
        let cmd = format!("{}\n", cmd);
        self.stream.get_mut().write_all(cmd.as_bytes())
    }
}
