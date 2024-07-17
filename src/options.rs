use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::event_handler;

use crate::commands::link;
use crate::commands::ping;
use crate::commands::profile;
use crate::commands::recent;
use crate::commands::top;

type CommandFn = for<'a> fn(
    &'a Context,                // Command context, `ctx`
    &'a Message,                // Message variable, `msg`
    Vec<&'a str>,               // Arguments of the command, `args`
    &'a event_handler::Handler, // The handler, `handler`
    &'a str,                    // The command's name, `command_name`
    Option<&'a str>,            // The command's alias (if it was passed), `command_alias`
    Option<usize>,              // The play index, `play_index`
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
            exec: |ctx, msg, args, handler, command_name, command_alias, _play_index| {
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
            exec: |ctx, msg, args, handler, command_name, command_alias, _play_index| {
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
            exec: |ctx, msg, args, handler, command_name, command_alias, _play_index| {
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
        Command {
            name: "recent",
            aliases: vec![
                // osu! Standard
                "recent",
                "rs",
                "r",
                "recentpass",
                "rp",
                // osu! Mania
                "recentmania",
                "rm",
                "recentmaniapass",
                "rmp",
                // osu! Taiko
                "recenttaiko",
                "rt",
                "recenttaikopass",
                "rtp",
                // osu! Catch
                "recentcatch",
                "rc",
                "recentcatchpass",
                "rcp",
            ],
            exec: |ctx, msg, args, handler, command_name, command_alias, play_index| {
                Box::pin(recent::execute(
                    ctx,
                    msg,
                    args,
                    handler,
                    command_name,
                    command_alias,
                    play_index,
                ))
            },
        },
        Command {
            name: "top",
            aliases: vec![
                "top", "t", // osu! Standard
                "topmania", "topm", "tm", // osu! Mania
                "toptaiko", "topt", "tt", // osu! Taiko
                "topcatch", "topc", "tc", // osu! Catch
            ],
            exec: |ctx, msg, args, handler, command_name, command_alias, play_index| {
                Box::pin(top::execute(
                    ctx,
                    msg,
                    args,
                    handler,
                    command_name,
                    command_alias,
                    play_index,
                ))
            },
        },
    ]
}
