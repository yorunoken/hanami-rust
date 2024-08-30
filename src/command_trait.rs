use serenity::{
    all::{CommandInteraction, Context, Message},
    async_trait,
    builder::CreateCommand,
    Error,
};

#[async_trait]
pub trait Command {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn run(
        &self,
        ctx: &Context,
        msg: &Message,
        args: Vec<&str>,
        command: &str,
    ) -> Result<(), Error>;

    async fn run_slash(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), Error>;

    fn register(&self) -> CreateCommand;
}
