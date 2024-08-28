use crate::{command_trait::Command, commands::ping::Ping};

pub fn get_commands() -> Vec<Box<dyn Command + Send + Sync>> {
    vec![Box::new(Ping)]
}
