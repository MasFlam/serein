use std::collections::HashMap;

use darling::{
	FromDeriveInput, FromField, FromVariant,
	ast::{Data, Fields},
	util::Ignored,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(serein), supports(enum_newtype))]
struct RootOpts {
	pub data: Data<VariantOpts, Ignored>,
}

#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(serein))]
struct VariantOpts {
	pub ident: Ident,
	pub fields: Fields<VariantFieldOpts>,

	pub name: Option<String>,
	pub desc: String,

	#[darling(default)]
	pub names: HashMap<String, String>,

	#[darling(default)]
	pub descs: HashMap<String, String>,

	#[darling(default)]
	pub nsfw: bool,
}

impl VariantOpts {
	pub fn name(&self) -> String {
		self.name
			.clone()
			.unwrap_or_else(|| self.ident.to_string().to_lowercase())
	}

	pub fn ty(&self) -> &Type {
		&self.fields.fields[0].ty
	}
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(serein))]
struct VariantFieldOpts {
	pub ty: Type,
}

pub fn derive(input: DeriveInput) -> TokenStream {
	let root = match RootOpts::from_derive_input(&input) {
		Ok(root) => root,
		Err(err) => return err.write_errors(),
	};

	let variants = root.data.take_enum().unwrap();

	let fn_dispatch = generate_dispatch(&variants, &input);
	let fn_create = generate_create(&variants, &input);

	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		#[::serenity::async_trait]
		impl #impl_generics ::serein::slash::CommandTree for #ident #ty_generics #where_clause {
			#fn_dispatch
			#fn_create
		}
	}
}

fn generate_dispatch(variants: &[VariantOpts], input: &DeriveInput) -> TokenStream {
	let match_arms = {
		let mut match_arms = Vec::<TokenStream>::new();

		for variant in variants {
			let name = variant.name();
			let ty = variant.ty();

			let arm = quote! {
				#name => <#ty as ::serein::slash::Command>::dispatch(ctx, int).await
			};

			match_arms.push(arm);
		}

		match_arms
	};

	quote! {
		async fn dispatch(ctx: ::serenity::all::Context, int: ::serenity::all::CommandInteraction) -> ::serein::Result<()> {
			let command_name = int.data.name.as_str();

			match command_name {
				#(#match_arms,)*
				_ => ::serein::Result::Err(::serein::error::Error::UnrecognizedCommand),
			}
		}
	}
}

fn generate_create(variants: &[VariantOpts], input: &DeriveInput) -> TokenStream {
	let creates = {
		let mut creates = Vec::<TokenStream>::new();

		for variant in variants {
			let name = variant.name();
			let ty = variant.ty();

			let create = quote! {
				<#ty as ::serein::slash::Command>::create(#name)
			};

			creates.push(create);
		}

		creates
	};

	quote! {
		fn create() -> Vec<::serenity::all::CreateCommand> {
			vec![
				#(#creates,)*
			]
		}
	}
}
