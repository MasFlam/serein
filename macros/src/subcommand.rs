use darling::{FromDeriveInput, ast::Data};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::common::{FieldOpts, VariantOpts, generate_sub_or_subsub_create_from_struct};

#[derive(FromDeriveInput)]
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
			let fn_dispatch = generate_dispatch_from_enum(&variants);
			let fn_create = generate_create_from_enum(&variants);

			(fn_dispatch, fn_create)
		}
		Data::Struct(fields) => {
			let fn_dispatch = generate_dispatch_from_struct(&fields.fields);
			let fn_create = generate_create_from_struct(&fields.fields);

			(fn_dispatch, fn_create)
		}
	};

	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		#[::serenity::async_trait]
		impl #impl_generics ::serein::slash::SubCommand for #ident #ty_generics #where_clause {
			#fn_dispatch
			#fn_create
		}
	}
}

fn generate_dispatch_from_enum(variants: &[VariantOpts]) -> TokenStream {
	let match_arms = {
		let mut match_arms = Vec::<TokenStream>::new();

		for variant in variants {
			let name = variant.name();
			let ty = variant.ty();

			let arm = quote! {
				#name => <#ty as ::serein::slash::SubSubCommand>::dispatch(ctx, int).await
			};

			match_arms.push(arm);
		}

		match_arms
	};

	quote! {
		async fn dispatch(ctx: ::serenity::all::Context, int: ::serenity::all:CommandInteraction) -> ::serein::Result<()> {
			if int.data.options.len() != 1 {
				return ::serein::Result::Err(::serein::Error::UnrecognizedCommand);
			}

			let opt = &int.data.options[0];

			match &opt.value {
				::serenity::all::CommandDataOptionValue::SubCommandGroup(sub_opts) => {
					if sub_opts.len() != {
						return ::serein::Result::Err(::serein::Error::UnrecognizedCommand);
					}

					let sub_opt = &sub_opts[0];

					match sub_opt.name.as_str() {
						#(#match_arms,)*
						_ => ::serein::Result::Err(::serein::Error::UnrecognizedCommand),
					}
				},
				_ => {
					::serein::Result::Err(::serein::Error::UnrecognizedCommand)
				}
			}
		}
	}
}

fn generate_dispatch_from_struct(fields: &[FieldOpts]) -> TokenStream {
	let self_fields = {
		let mut self_fields = Vec::<TokenStream>::new();

		for field in fields {
			let ident = field.ident.as_ref().unwrap();
			let name = field.name();
			let ty = &field.ty;

			let on_missing = if field.default.is_present() {
				quote! {
					<#ty as Default>::default()
				}
			} else {
				quote! {
					<#ty as ::serein::options::CommandOption>::try_from_missing_value()?
				}
			};

			let self_field = quote! {
				#ident: match sub_opts.iter().filter(|opt| opt.name == #name).last() {
					Some(opt) => {
						<#ty as ::serein::options::CommandOption>::try_from_resolved_value(opt.value.clone())?
					}
					None => {
						#on_missing
					}
				}
			};

			self_fields.push(self_field);
		}

		self_fields
	};

	quote! {
		async fn dispatch(ctx: ::serenity::all::Context, int: ::serenity::all:CommandInteraction) -> ::serein::Result<()> {
			if int.data.options.len() != 1 {
				return ::serein::Result::Err(::serein::Error::UnrecognizedCommand);
			}

			let opt = &int.data.options[0];

			match &opt.value {
				::serenity::all::CommandDataOptionValue::SubCommandGroup(sub_opts) => {
					let obj = Self {
						#(#self_fields)*
					};

					obj.handle(ctx, int).await
				},
				_ => {
					::serein::Result::Err(::serein::Error::UnrecognizedCommand)
				}
			}
		}
	}
}

fn generate_create_from_enum(variants: &[VariantOpts]) -> TokenStream {
	let sub_opt_creates = {
		let mut sub_opt_creates = Vec::<TokenStream>::new();

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
				<#ty as ::serein::slash::SubSubCommand>::create(#name, #desc)
					#(#dot_names)*
					#(#dot_descs)*
			};

			sub_opt_creates.push(create);
		}

		sub_opt_creates
	};

	quote! {
		fn create(name: impl Into<String>, desc: impl Into<String>) -> ::serenity::all::CreateCommandOption {
			::serenity::all::CreateCommandOption::new(
				::serenity::all::CommandOptionType::SubCommandGroup,
				name,
				desc,
			)
			.set_sub_options(vec![
				#(#sub_opt_creates,)*
			])
		}
	}
}

fn generate_create_from_struct(fields: &[FieldOpts]) -> TokenStream {
	generate_sub_or_subsub_create_from_struct(fields)
}
