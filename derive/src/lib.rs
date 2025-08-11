use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::choices::ChoiceKind;

pub(crate) mod choices;
pub(crate) mod command;
pub(crate) mod command_tree;
pub(crate) mod common;
pub(crate) mod subcommand;
pub(crate) mod subsubcommand;

#[proc_macro_derive(CommandTree, attributes(serein))]
pub fn derive_command_tree(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	command_tree::derive(input).into()
}

#[proc_macro_derive(Command, attributes(serein))]
pub fn derive_command(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	command::derive(input).into()
}

#[proc_macro_derive(SubCommand, attributes(serein))]
pub fn derive_subcommand(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	subcommand::derive(input).into()
}

#[proc_macro_derive(SubSubCommand, attributes(serein))]
pub fn derive_subsubcommand(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	subsubcommand::derive(input).into()
}

#[proc_macro_derive(StringChoice, attributes(serein))]
pub fn derive_string_choice(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	choices::derive(input, ChoiceKind::String).into()
}

#[proc_macro_derive(IntChoice, attributes(serein))]
pub fn derive_int_choice(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	choices::derive(input, ChoiceKind::Int).into()
}

#[proc_macro_derive(FloatChoice, attributes(serein))]
pub fn derive_float_choice(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	choices::derive(input, ChoiceKind::Float).into()
}
