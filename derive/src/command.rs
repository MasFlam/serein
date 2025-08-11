use darling::{FromDeriveInput, ast::Data};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::common::{FieldOpts, VariantOpts};

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(attributes(serein), supports(enum_newtype, struct_named, struct_unit))]
struct RootOpts {
	pub data: Data<VariantOpts, FieldOpts>,
}

pub fn derive(input: DeriveInput) -> TokenStream {
	let root = match RootOpts::from_derive_input(&input) {
		Ok(root) => root,
		Err(err) => return err.write_errors(),
	};

	let (fn_dispatch, fn_create) = match root.data {
		Data::Enum(variants) => {
			let fn_dispatch = generate_dispatch_from_enum(&variants, &input);
			let fn_create = generate_create_from_enum(&variants, &input);

			(fn_dispatch, fn_create)
		}
		Data::Struct(fields) => {
			let fn_dispatch = generate_dispatch_from_struct(&fields.fields, &input);
			let fn_create = generate_create_from_struct(&fields.fields, &input);

			(fn_dispatch, fn_create)
		}
	};

	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		#[::serenity::async_trait]
		impl #impl_generics ::serein::slash::Command for #ident #ty_generics #where_clause {
			#fn_dispatch
			#fn_create
		}
	}
}

fn generate_dispatch_from_enum(variants: &[VariantOpts], input: &DeriveInput) -> TokenStream {
	let match_arms = {
		let mut match_arms = Vec::<TokenStream>::new();

		for variant in variants {
			let name = variant.name();
			let ty = variant.ty();

			let arm = quote! {
				#name => <#ty as ::serein::slash::SubCommand>::dispatch(ctx, int).await
			};

			match_arms.push(arm);
		}

		match_arms
	};

	quote! {
		async fn dispatch(ctx: ::serenity::all::Context, int: ::serenity::all:CommandInteraction) -> ::serein::Result<()> {
			if int.data.options.len() != 1 {
				return ::serein::Result::Err(::serein::error::Error::UnrecognizedCommand);
			}

			let opt = &int.data.options[0];

			match opt.kind() {
				::serenity::all::CommandOptionType::SubCommand | ::serenity::all::CommandOptionType::SubCommandGroup => {
					match opt.name.as_str() {
						#(#match_arms,)*
						_ => ::serein::Result::Err(::serein::error::Error::UnrecognizedCommand),
					}
				},
				_ => {
					::serein::Result::Err(::serein::error::Error::UnrecognizedCommand)
				}
			}
		}
	}
}

fn generate_dispatch_from_struct(fields: &[FieldOpts], input: &DeriveInput) -> TokenStream {
	let self_fields = {
		let mut self_fields = Vec::<TokenStream>::new();

		for field in fields {
			let ident = field.ident.as_ref().unwrap();
			let name = field.name();
			let ty = &field.ty;
			let self_field = quote! {
				#ident: match opts.iter().filter(|opt| opt.name == #name).last() {
					Some(opt) => {
						<#ty as ::serein::options::CommandOption>::try_from_resolved_value(opt.value.clone())?
					}
					None => {
						<#ty as ::serein::options::CommandOption>::try_from_missing_value()?
					}
				}
			};

			self_fields.push(self_field);
		}

		self_fields
	};

	quote! {
		async fn dispatch(ctx: ::serenity::all::Context, int: ::serenity::all::CommandInteraction) -> ::serein::Result<()> {
			let opts = int.data.options();

			let obj = Self {
				#(#self_fields,)*
			};

			obj.handle(ctx, int).await
		}
	}
}

fn generate_create_from_enum(variants: &[VariantOpts], input: &DeriveInput) -> TokenStream {
	let opt_creates = {
		let mut opt_creates = Vec::<TokenStream>::new();

		for variant in variants {
			let name = variant.name();
			let ty = variant.ty();
			let desc = &variant.desc;

			let dot_names: Vec<TokenStream> = variant
				.names
				.iter()
				.map(|(locale, string)| quote! { .name_localized(#locale, #string) })
				.collect();

			let dot_descs: Vec<TokenStream> = variant
				.descs
				.iter()
				.map(|(locale, string)| quote! { .description_localized(#locale, #string) })
				.collect();

			let create = quote! {
				<#ty as ::serein::slash::SubCommand>::create(#name, #desc)
					#(#dot_names)*
					#(#dot_descs)*
			};

			opt_creates.push(create);
		}

		opt_creates
	};

	quote! {
		fn create(name: impl Into<String>) -> ::serenity::all::CreateCommand {
			::serenity::all::CreateCommand::new(name)
				.kind(::serenity::all::CommandType::ChatInput)
				.set_options(vec![
					#(#opt_creates,)*
				])
		}
	}
}

fn generate_create_from_struct(fields: &[FieldOpts], input: &DeriveInput) -> TokenStream {
	let opt_creates = {
		let mut opt_creates = Vec::<TokenStream>::new();

		for field in fields {
			let name = field.name();
			let ty = &field.ty;
			let desc = &field.desc;

			let dot_names: Vec<TokenStream> = field
				.names
				.iter()
				.map(|(locale, string)| quote! { .name_localized(#locale, #string) })
				.collect();

			let dot_descs: Vec<TokenStream> = field
				.descs
				.iter()
				.map(|(locale, string)| quote! { .description_localized(#locale, #string) })
				.collect();

			let create = quote! {
				<#ty as ::serein::options::CommandOption>::create(#name, #desc)
					#(#dot_names)*
					#(#dot_descs)*
			};

			opt_creates.push(create);
		}

		opt_creates
	};

	quote! {
		fn create(name: impl Into<String>) -> ::serenity::all::CreateCommand {
			::serenity::all::CreateCommand::new(name)
				.kind(::serenity::all::CommandType::ChatInput)
				.set_options(vec![
					#(#opt_creates,)*
				])
		}
	}
}
