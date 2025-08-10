use darling::{FromDeriveInput, ast::Data, util::Ignored};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::DeriveInput;

use crate::{ChoiceKind, common::ChoiceOpts};

#[derive(FromDeriveInput)]
#[darling(attributes(serein), supports(enum_unit))]
struct RootOpts {
	pub data: Data<ChoiceOpts, Ignored>,
}

pub fn derive(input: DeriveInput, kind: ChoiceKind) -> TokenStream {
	let root = match RootOpts::from_derive_input(&input) {
		Ok(root) => root,
		Err(err) => return err.write_errors(),
	};

	let (choice_trait, value_type, variant_name, add_fn) = match kind {
		ChoiceKind::String => (
			format_ident!("StringChoice"),
			quote!(&str),
			format_ident!("String"),
			format_ident!("add_string_choice_localized"),
		),
		ChoiceKind::Int => (
			format_ident!("IntChoice"),
			quote!(i64),
			format_ident!("Integer"),
			format_ident!("add_int_choice_localized"),
		),
		ChoiceKind::Float => (
			format_ident!("FloatChoice"),
			quote!(f64),
			format_ident!("Number"),
			format_ident!("add_number_choice_localized"),
		),
	};

	let choices = root.data.take_enum().unwrap();
	let mut match_arms = Vec::<TokenStream>::new();
	let mut dot_choices = Vec::<TokenStream>::new();

	for choice in &choices {
		let ChoiceOpts {
			ident,
			discriminant,
			name,
			names,
			value,
		} = choice;

		let name = name.to_owned().unwrap_or_else(|| ident.to_string());

		let name_localizations: Vec<TokenStream> = names
			.iter()
			.map(|(locale, string)| {
				let locale = locale.replace('_', "-");
				quote! {
					(#locale, #string)
				}
			})
			.collect();

		let value = match kind {
			ChoiceKind::String => value
				.as_ref()
				.map(|value| value.to_token_stream())
				.or_else(|| Some(ident.to_string().to_token_stream())),
			ChoiceKind::Int => value
				.as_ref()
				.map(|value| value.to_token_stream())
				.or_else(|| discriminant.as_ref().map(|d| d.to_token_stream())),
			ChoiceKind::Float => value.as_ref().map(|value| value.to_token_stream()),
		};

		let value = match value {
			Some(value) => value,
			None => {
				return syn::Error::new(ident.span(), "no choice value provided")
					.into_compile_error();
			}
		};

		let arm = quote! {
			_ if value == { #value } => Ok(Self::#ident)
		};

		let dot_choice = quote! {
			.#add_fn(
				#name,
				#value,
				[
					#(#name_localizations,)*
				],
			)
		};

		match_arms.push(arm);
		dot_choices.push(dot_choice);
	}

	let ident = &input.ident;
	let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

	let impl_choice_trait = quote! {
		impl #impl_generics ::serein::options::#choice_trait for #ident #type_generics #where_clause {
			fn from_value(value: #value_type) -> ::serein::Result<Self> {
				match value {
					#(#match_arms,)*
					_ => Err(::serein::error::Error::BadOptionValue),
				}
			}

			fn create_with_choices(name: ::alloc::String, desc: ::alloc::String) -> ::serenity::all::CreateCommandOption {
				::serenity::all::CreateCommandOption::new(
					::serenity::all::CommandOptionType::#variant_name,
					name,
					desc,
				)
				.required(true)
				#(#dot_choices)*
			}
		}
	};

	let impl_option_trait = quote! {
		impl #impl_generics ::serein::options::CommandOption for #ident #type_generics #where_clause {
			fn try_from_resolved_value(value: ::serenity::all::ResolvedValue) -> ::serein::Result<Self> {
				match value {
					ResolvedValue::#variant_name(value) => {
						<Self as ::serein::options::#choice_trait>::from_value(value)
					}
					_ => Err(::serein::error::Error::BadOptionType)
				}
			}

			fn create(name: ::alloc::String, desc: ::alloc::String) -> ::serenity::all::CreateCommandOption {
				<Self as ::serein::option::#choice_trait>::create_with_choices(name, desc)
			}
		}
	};

	quote! {
		#impl_choice_trait
		#impl_option_trait
	}
}
