use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::utils;

use crate::commands::link;
use crate::commands::ping;
use crate::commands::profile;

type CommandFn = for<'a> fn(
    &'a Context,                       // Command context, `ctx`
    &'a Message,                       // Message variable, `msg`
    Vec<&'a str>,                      // Arguments of the command, `args`
    &'a utils::event_handler::Handler, // The handler, `handler`
    &'a str,                           // The command's name, `command_name`
    Option<&'a str>,                   // The command's alias (if it was passed), `command_alias`
) -> BoxFuture<'a, Result<(), Error>>;

pub struct Command {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub exec: CommandFn,
}

// Define new commands here
pub fn get_prefix_commands() -> Vec<Command> {
    vec![
        Command {
            name: "ping",
            aliases: vec!["ping"],
            exec: |ctx, msg, args, handler, command_name, command_alias| {
                Box::pin(ping::execute(
                    ctx,
                    msg,
                    args,
                    handler,
                    command_name,
                    command_alias,
                ))
            },
        },
        Command {
            name: "profile",
            aliases: vec!["osu", "mania", "taiko", "catch"],
            exec: |ctx, msg, args, handler, command_name, command_alias| {
                Box::pin(profile::execute(
                    ctx,
                    msg,
                    args,
                    handler,
                    command_name,
                    command_alias,
                ))
            },
        },
        Command {
            name: "link",
            aliases: vec!["link"],
            exec: |ctx, msg, args, handler, command_name, command_alias| {
                Box::pin(link::execute(
                    ctx,
                    msg,
                    args,
                    handler,
                    command_name,
                    command_alias,
                ))
            },
        },
    ]
}
