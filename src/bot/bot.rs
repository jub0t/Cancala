use serde::Serialize;
use std::fs;
use std::process::{Child, Command, Stdio};
use uuid::Uuid;

use crate::bot::io::IndependantIO;
use crate::utils::thead::to_arc_mutex;

use super::io::SafeIoSender;
use super::manager::BotEngine;

#[derive(Serialize)]
pub enum BotStatus {
    Stopped,
    Running,
    Paused,
    None,
}

impl BotStatus {
    pub fn as_uint32(&self) -> u32 {
        match self {
            Self::None => {
                return 0;
            }

            Self::Running => {
                return 1;
            }

            Self::Stopped => {
                return 2;
            }

            Self::Paused => {
                return 3;
            }
        }
    }
}

pub type BotId = String;

pub struct Bot {
    pub id: BotId,
    pub name: String,
    pub process: Option<Child>,

    // Option<String> for now i guess.
    pub absolute_path: Option<String>,
    pub engine: BotEngine,

    pub status: BotStatus,
}

pub struct StartBotOptions {
    pub io_sender: SafeIoSender,
}

impl Bot {
    pub fn new(name: &str) -> Self {
        let id = Uuid::new_v4();

        Bot {
            id: id.to_string(),
            name: name.to_string(),
            engine: BotEngine::Node,
            status: BotStatus::None,
            absolute_path: None,
            process: None,
        }
    }

    pub fn initialize(&self) -> bool {
        // Initialize a directory if absolute path exists
        match self.absolute_path.as_ref() {
            None => {
                return false;
            }
            Some(path) => {
                let result = fs::create_dir(path);
                return result.is_ok();
            }
        };
    }

    // Start the bot process
    pub fn start(
        &mut self,
        arguments: Vec<String>, // TODO: maybe we can use a better data type?
        options: StartBotOptions,
    ) -> std::io::Result<()> {
        if self.absolute_path == None {
            // TODO: Implement an error for this.
            return Ok(());
        }

        if self.process.is_none() {
            let engine_cmd = self.engine.as_string();
            let mut child = Command::new(engine_cmd);

            // Add all arguments
            for arg in arguments {
                child.arg(arg);
            }

            let mut spawned = child
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let stdout = spawned.stdout.take().unwrap();
            let safe_out = to_arc_mutex(stdout);

            let stderr = spawned.stderr.take().unwrap();
            let safe_err = to_arc_mutex(stderr);

            let bot_io = IndependantIO::new(safe_out, safe_err, options.io_sender);
            bot_io.activate();

            self.process = Some(spawned);
            println!("Bot {} started", self.name);
        }

        Ok(())
    }

    // Stop the bot process
    pub fn stop(&mut self) -> std::io::Result<()> {
        if let Some(ref mut process) = self.process {
            process.kill()?;
            self.process = None;
        }

        Ok(())
    }
}
