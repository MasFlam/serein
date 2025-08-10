use darling::{FromDeriveInput, ast::Data};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::common::{
	FieldOpts, VariantOpts, derive_from_enum, derive_from_struct, make_option_creates,
	make_sub_option_creates,
};

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

	match root.data {
		Data::Enum(variants) => {
			let a = derive_from_enum(
				&variants,
				&input,
				format_ident!("Command"),
				format_ident!("SubCommand"),
			);
			let b = derive_create(
				make_sub_option_creates(&variants, format_ident!("SubCommandCreate")),
				&input,
			);

			quote! {
				#a
				#b
			}
		}
		Data::Struct(fields) => {
			let a = derive_from_struct(&fields.fields, &input, format_ident!("Command"));
			let b = derive_create(make_option_creates(&fields.fields), &input);

			quote! {
				#a
				#b
			}
		}
	}
}

fn derive_create(creates: Vec<TokenStream>, input: &DeriveInput) -> TokenStream {
	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		impl #impl_generics ::serein::slash::CommandCreate for #ident #ty_generics #where_clause {
			fn create(name: &str) -> ::serenity::all::CreateCommand {
				::serenity::all::CreateCommand::new(name)
					.kind(::serenity::all::CommandType::ChatInput)
					.set_options(vec![
						#(#creates,)*
					])
			}
		}
	}
}
