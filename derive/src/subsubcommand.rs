use darling::{FromDeriveInput, ast::Data, util::Ignored};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::common::FieldOpts;

#[derive(FromDeriveInput)]
#[darling(attributes(serein), supports(enum_newtype, struct_named, struct_unit))]
struct RootOpts {
	pub data: Data<Ignored, FieldOpts>,
}

pub fn derive(input: DeriveInput) -> TokenStream {
	let root = match RootOpts::from_derive_input(&input) {
		Ok(root) => root,
		Err(err) => return err.write_errors(),
	};

	let fields = root.data.take_struct().unwrap();

	let fn_dispatch = generate_dispatch(&fields.fields, &input);
	let fn_create = generate_create(&fields.fields, &input);

	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		#[::serenity::async_trait]
		impl #impl_generics ::serein::slash::SubSubCommand for #ident #ty_generics #where_clause {
			#fn_dispatch
			#fn_create
		}
	}
}

fn generate_dispatch(fields: &[FieldOpts], input: &DeriveInput) -> TokenStream {
	let self_fields = {
		let mut self_fields = Vec::<TokenStream>::new();

		for field in fields {
			let ident = field.ident.as_ref().unwrap();
			let name = field.name();
			let ty = &field.ty;

			let self_field = quote! {
				#ident: match sub_sub_opts.iter().filter(|opt| opt.name == #name).last() {
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
		async fn dispatch(ctx: ::serenity::all::Context, int: ::serenity::all:CommandInteraction) -> ::serein::Result<()> {
			if int.data.options.len() != 1 {
				return ::serein::Result::Err(::serein::error::Error::UnrecognizedCommand);
			}

			let opt = &int.data.options[0];

			match &opt.value {
				::serenity::all::CommandDataOptionValue::SubCommandGroup(sub_opts) => {
					if sub_opts.len() != 1 {
						return ::serein::Result::Err(::serein::error::Error::UnrecognizedCommand);
					}

					let sub_opt = &sub_opts[0];

					match &sub_opt.value {
						::serenity::all::CommandDataOptionValue::SubCommand(sub_sub_opts) => {
							let obj = Self {
								#(#self_fields)*
							};

							obj.handle(ctx, int).await
						}
					}
				},
				_ => {
					::serein::Result::Err(::serein::error::Error::UnrecognizedCommand)
				}
			}
		}
	}
}

fn generate_create(fields: &[FieldOpts], input: &DeriveInput) -> TokenStream {
	let sub_opt_creates = {
		let mut sub_opt_creates = Vec::<TokenStream>::new();

		for field in fields {
			let name = field.name();
			let ty = &field.ty;
			let desc = &field.desc;

			let dot_names: Vec<TokenStream> = field
				.names
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! { .name_localized(#locale, #string) }
				})
				.collect();

			let dot_descs: Vec<TokenStream> = field
				.descs
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! { .description_localized(#locale, #string) }
				})
				.collect();

			let create = quote! {
				<#ty as ::serein::options::CommandOption>::create(#name, #desc)
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
				::serenity::all::CommandOptionType::SubCommand,
				name,
				desc,
			)
			.set_sub_options(vec![
				#(#sub_opt_creates,)*
			])
		}
	}
}
