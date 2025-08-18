use darling::{FromDeriveInput, ast::Data, util::Ignored};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::common::{FieldOpts, generate_sub_or_subsub_create_from_struct};

#[derive(FromDeriveInput)]
#[darling(attributes(serein), supports(enum_newtype, struct_named, struct_unit))]
struct RootOpts {
	data: Data<Ignored, FieldOpts>,
}

pub fn derive(input: DeriveInput) -> TokenStream {
	let root = match RootOpts::from_derive_input(&input) {
		Ok(root) => root,
		Err(err) => return err.write_errors(),
	};

	let fields = root.data.take_struct().unwrap();

	let fn_dispatch = generate_dispatch(&fields.fields);
	let fn_create = generate_create(&fields.fields);

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

fn generate_dispatch(fields: &[FieldOpts]) -> TokenStream {
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
				#ident: match sub_sub_opts.iter().filter(|opt| opt.name == #name).last() {
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
		async fn dispatch(ctx: ::serenity::all::Context, int: ::serenity::all::Interaction) -> ::serein::Result<()> {
			let cint = match &int {
				::serenity::all::Interaction::Autocomplete(i) => i,
				::serenity::all::Interaction::Command(i) => i,
				_ => return ::serein::Result::Err(::serein::Error::UnrecognizedCommand),
			};

			if cint.data.options.len() != 1 {
				return ::serein::Result::Err(::serein::Error::UnrecognizedCommand);
			}

			let opt = &cint.data.options[0];

			match &opt.value {
				::serenity::all::CommandDataOptionValue::SubCommandGroup(sub_opts) => {
					if sub_opts.len() != 1 {
						return ::serein::Result::Err(::serein::Error::UnrecognizedCommand);
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
					::serein::Result::Err(::serein::Error::UnrecognizedCommand)
				}
			}
		}
	}
}

fn generate_create(fields: &[FieldOpts]) -> TokenStream {
	generate_sub_or_subsub_create_from_struct(fields)
}
