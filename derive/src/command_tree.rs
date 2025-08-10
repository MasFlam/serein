use darling::{FromDeriveInput, ast::Data, util::Ignored};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::common::{
	TopLevelOpts, VariantOpts, derive_from_enum, make_top_level_creates, touch_all_variants,
};

#[derive(FromDeriveInput)]
#[darling(attributes(serein), supports(enum_newtype))]
struct RootOpts {
	pub data: Data<TopLevelOpts, Ignored>,
}

pub fn derive(input: DeriveInput) -> TokenStream {
	let root = match RootOpts::from_derive_input(&input) {
		Ok(root) => root,
		Err(err) => return err.write_errors(),
	};

	let variants_toplevel = root.data.take_enum().unwrap();
	let variants: Vec<VariantOpts> = variants_toplevel.iter().cloned().map(Into::into).collect();

	let a = derive_from_enum(
		&variants,
		&input,
		format_ident!("CommandTree"),
		format_ident!("Command"),
	);
	let b = derive_create(
		make_top_level_creates(&variants_toplevel, format_ident!("CommandCreate")),
		&input,
	);
	let c = touch_all_variants(&input);

	quote! {
		#a
		#b
		#c
	}
}

fn derive_create(creates: Vec<TokenStream>, input: &DeriveInput) -> TokenStream {
	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		impl #impl_generics ::serein::slash::CommandTreeCreate for #ident #ty_generics #where_clause {
			fn create() -> Vec<::serenity::all::CreateCommand> {
				vec![
					#(#creates,)*
				]
			}
		}
	}
}
