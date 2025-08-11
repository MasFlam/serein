use async_trait::async_trait;
use serenity::all::{CommandInteraction, Context, CreateCommand, CreateCommandOption};

use crate::error::Result;

pub use serein_derive::{Command, CommandTree, SubCommand, SubSubCommand};

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

#[allow(unused)]
mod shite {
	use serein_derive::{Command, CommandTree};
	use serenity::all::{CommandInteraction, CommandOptionType, Context, ResolvedValue};

	use crate::slash::CommandHandler;

	#[derive(CommandTree)]
	enum Tree {
		#[serein(desc = "Greet someone", names(pl_PL = "cześć"))]
		Hello(CmdHello),
	}

	#[derive(Command)]
	struct CmdHello {
		#[serein(
			name = "who",
			names(pl_PL = "kto"),
			desc = "Who to greet",
			descs(pl_PL = "Kogo przywitać")
		)]
		name: String,
	}

	#[serenity::async_trait]
	impl CommandHandler for CmdHello {
		async fn handle(
			&self,
			ctx: Context,
			interaction: CommandInteraction,
		) -> serein::Result<()> {
			let x = Some(5);

			if interaction.data.options.len() != 1 {
				return ::serein::Result::Err(::serein::error::Error::UnrecognizedCommand);
			}

			let opt = &interaction.data.options[0];

			todo!()
		}
	}
}
