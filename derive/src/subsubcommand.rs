use darling::{FromDeriveInput, ast::Data, util::Ignored};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident};

use crate::common::{FieldOpts, derive_from_struct, make_option_creates};

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

	let a = derive_from_struct(&fields.fields, &input, format_ident!("SubSubCommand"));
	let b = derive_create(
		make_option_creates(&fields.fields),
		format_ident!("SubCommand"),
		&input,
	);

	quote! {
		#a
		#b
	}
}

fn derive_create(
	creates: Vec<TokenStream>,
	option_type: Ident,
	input: &DeriveInput,
) -> TokenStream {
	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		impl #impl_generics ::serein::slash::SubSubCommandCreate for #ident #ty_generics #where_clause {
			fn create(name: &str) -> ::serenity::all::CreateCommandOption {
				::serenity::all::CreateCommandOption::new(
					::serenity::all::CommandOptionType::#option_type,
					name,
					""
				)
				.set_options(vec![
					#(#creates,)*
				])
			}
		}
	}
}
