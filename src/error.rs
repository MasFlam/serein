use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("serenity error")]
	Serenity(#[from] serenity::Error),
	#[error("unrecognized command")]
	UnrecognizedCommand,
	#[error("missing option")]
	MissingOption,
	#[error("bad option type")]
	BadOptionType,
	#[error("bad option value")]
	BadOptionValue,
}

pub type Result<T> = core::result::Result<T, Error>;
