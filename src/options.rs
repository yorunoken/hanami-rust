use crate::command_trait::Command;

use crate::commands::general::ping::Ping;
use crate::commands::osu::profile::Profile;

pub fn get_commands() -> Vec<Box<dyn Command + Send + Sync>> {
    vec![
        // General
        Box::new(Ping),
        // Osu/Profile
        Box::new(Profile),
    ]
}
