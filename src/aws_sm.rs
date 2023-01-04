use std::{
    convert::AsRef,
    ffi::OsStr,
    process::{exit, Command},
    str::from_utf8,
};

pub struct AwsSM {
    command: Command,
}

impl AwsSM {
    pub fn new(subcommand: &str) -> Self {
        let mut command = Command::new("aws");
        command.arg("secretsmanager").arg(subcommand);

        AwsSM { command }
    }

    /// Run the command and return the output as a String. Exits with status code 1 if there is an
    /// error running the command.
    pub fn run(mut self) -> String {
        let output = match self.command.output() {
            Ok(o) => {
                let stdout = from_utf8(&o.stdout)
                    .expect("Failed to parse stdout")
                    .to_string();
                let stderr = from_utf8(&o.stderr)
                    .expect("Failed to parse stderr")
                    .to_string();
                if !o.status.success() {
                    eprintln!("Failed to run aws command.\n{}", stderr);
                    exit(1)
                }
                stdout
            }
            Err(e) => {
                eprintln!(
                    "Failed to run aws command. Is the AWS CLI installed?\n{}",
                    e
                );
                exit(1);
            }
        };
        output
    }

    /// Append a series of arguments to the command
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }
}
