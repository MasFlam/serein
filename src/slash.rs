use async_trait::async_trait;
use serenity::all::{CommandInteraction, Context, CreateCommand, CreateCommandOption};

use crate::error::Result;

pub use serein_macros::{Command, CommandTree, SubCommand, SubSubCommand};

#[async_trait]
pub trait CommandTree {
	async fn dispatch(ctx: Context, int: CommandInteraction) -> Result<()>;
	fn create() -> Vec<CreateCommand>;
}

#[async_trait]
pub trait Command {
	async fn dispatch(ctx: Context, int: CommandInteraction) -> Result<()>;
	fn create(name: impl Into<String>) -> CreateCommand;
}

#[async_trait]
pub trait SubCommand {
	async fn dispatch(ctx: Context, int: CommandInteraction) -> Result<()>;
	fn create(name: impl Into<String>, desc: impl Into<String>) -> CreateCommandOption;
}

#[async_trait]
pub trait SubSubCommand {
	async fn dispatch(ctx: Context, int: CommandInteraction) -> Result<()>;
	fn create(name: impl Into<String>, desc: impl Into<String>) -> CreateCommandOption;
}

#[async_trait]
pub trait CommandHandler {
	async fn handle(&self, ctx: Context, int: CommandInteraction) -> Result<()>;
}
