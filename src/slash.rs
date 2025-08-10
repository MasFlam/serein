use async_trait::async_trait;
use serenity::all::{CommandInteraction, Context, CreateCommand, CreateCommandOption};

use crate::error::Result;

pub use serein_derive::{Command, CommandTree, SubCommand, SubSubCommand};

#[async_trait]
pub trait CommandTree {
	async fn dispatch(ctx: Context, interaction: CommandInteraction) -> Result<()>;
}

pub trait CommandTreeCreate {
	fn create() -> Vec<CreateCommand>;
}

#[async_trait]
pub trait Command {
	async fn dispatch(ctx: Context, interaction: CommandInteraction) -> Result<()>;
}

pub trait CommandCreate {
	fn create(name: &str) -> CreateCommand;
}

#[async_trait]
pub trait SubCommand {
	async fn dispatch(ctx: Context, interaction: CommandInteraction) -> Result<()>;
}

pub trait SubCommandCreate {
	fn create(name: &str) -> CreateCommandOption;
}

#[async_trait]
pub trait SubSubCommand {
	async fn dispatch(ctx: Context, interaction: CommandInteraction) -> Result<()>;
}

pub trait SubSubCommandCreate {
	fn create(name: &str) -> CreateCommandOption;
}

#[async_trait]
pub trait Handler {
	async fn handle(&self, ctx: Context, interaction: CommandInteraction) -> Result<()>;
}

#[allow(unused)]
mod shite {
	use serein_derive::{Command, CommandTree};
	use serenity::all::{CommandInteraction, Context, ResolvedValue};

	use crate::slash::Handler;

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
	impl Handler for CmdHello {
		async fn handle(
			&self,
			ctx: Context,
			interaction: CommandInteraction,
		) -> serein::Result<()> {
			let x = Some(5);

			todo!()
		}
	}
}
