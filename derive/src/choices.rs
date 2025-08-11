use std::collections::HashMap;

use darling::{FromDeriveInput, FromMeta, FromVariant, ast::Data, util::Ignored};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Expr, Ident};

#[derive(Clone, Copy, Debug)]
pub enum ChoiceKind {
	String,
	Int,
	Float,
}

impl ChoiceKind {
	pub fn choice_trait(&self) -> TokenStream {
		match self {
			Self::String => quote!(StringChoice),
			Self::Int => quote!(IntChoice),
			Self::Float => quote!(FloatChoice),
		}
	}
	pub fn value_type(&self) -> TokenStream {
		match self {
			Self::String => quote!(&str),
			Self::Int => quote!(i64),
			Self::Float => quote!(f64),
		}
	}
	pub fn resolved_value_variant(&self) -> TokenStream {
		match self {
			Self::String => quote!(String),
			Self::Int => quote!(Integer),
			Self::Float => quote!(Number),
		}
	}
	pub fn option_type_variant(&self) -> TokenStream {
		self.resolved_value_variant()
	}
	pub fn add_fn(&self) -> TokenStream {
		match self {
			Self::String => quote!(add_string_choice_localized),
			Self::Int => quote!(add_int_choice_localized),
			Self::Float => quote!(add_number_choice_localized),
		}
	}
}

#[derive(FromDeriveInput)]
#[darling(attributes(serein), supports(enum_unit))]
struct RootOpts {
	pub data: Data<VariantOpts, Ignored>,
}

#[derive(FromVariant)]
#[darling(attributes(serein), map = Self::after)]
struct VariantOpts {
	pub ident: Ident,
	pub discriminant: Option<Expr>,

	pub name: Option<String>,
	pub value: Option<ChoiceValue>,

	#[darling(default)]
	pub names: HashMap<String, String>,
}

impl VariantOpts {
	fn after(mut self) -> Self {
		self.names = self
			.names
			.into_iter()
			.map(|(locale, string)| {
				let locale = locale.replace('_', "-");
				(locale, string)
			})
			.collect();

		self
	}

	pub fn name(&self) -> String {
		self.name
			.clone()
			.unwrap_or_else(|| self.ident.to_string().to_lowercase())
	}

	pub fn value(&self, kind: ChoiceKind) -> Result<TokenStream, TokenStream> {
		let value = self.value.as_ref().map(|value| match value {
			ChoiceValue::String(value) => quote!(#value),
			ChoiceValue::Int(value) => quote!(#value),
			ChoiceValue::Float(value) => quote!(#value),
		});

		match kind {
			ChoiceKind::String => value.ok_or_else(|| {
				syn::Error::new(self.ident.span(), "provide a value through an attribute")
					.into_compile_error()
			}),
			ChoiceKind::Int => value
				.or_else(|| {
					self.discriminant
						.as_ref()
						.map(|expr| expr.to_token_stream())
				})
				.ok_or_else(|| {
					syn::Error::new(
						self.ident.span(),
						"provide a value through an attribute or discriminant",
					)
					.into_compile_error()
				}),
			ChoiceKind::Float => value.ok_or_else(|| {
				syn::Error::new(self.ident.span(), "provide a value through an attribute")
					.into_compile_error()
			}),
		}
	}
}

#[derive(FromMeta)]
enum ChoiceValue {
	String(String),
	Int(i64),
	Float(f64),
}

pub fn derive(input: DeriveInput, kind: ChoiceKind) -> TokenStream {
	let root = match RootOpts::from_derive_input(&input) {
		Ok(root) => root,
		Err(err) => return err.write_errors(),
	};

	let variants = root.data.take_enum().unwrap();

	let fn_from_resolved_value = match generate_from_resolved_value(&variants, &input, kind) {
		Ok(f) => f,
		Err(err) => return err,
	};

	let fn_create = match generate_create(&variants, &input, kind) {
		Ok(f) => f,
		Err(err) => return err,
	};

	let choice_trait = kind.choice_trait();

	let ident = &input.ident;
	let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		impl #impl_generics ::serein::options::#choice_trait for #ident #type_generics #where_clause {}
		impl #impl_generics ::serein::options::CommandOption for #ident #type_generics #where_clause {
			#fn_from_resolved_value
			#fn_create
		}
	}
}

fn generate_from_resolved_value(
	variants: &[VariantOpts],
	input: &DeriveInput,
	kind: ChoiceKind,
) -> Result<TokenStream, TokenStream> {
	let resolved_value_variant = kind.resolved_value_variant();

	let match_arms = {
		let mut match_arms = Vec::<TokenStream>::new();

		for variant in variants {
			let ident = &variant.ident;
			let value = variant.value(kind)?;

			let arm = quote! {
				#value => Self::#ident
			};

			match_arms.push(arm);
		}

		match_arms
	};

	Ok(quote! {
		fn from_resolved_value(value: ::serenity::all::ResolvedValue) -> ::serein::Result<Self> {
			match value {
				::serenity::all::ResolvedValue::#resolved_value_variant(value) => {
					match value {
						#(#match_arms,)*
						_ => Err(::serein::error::Error::BadOptionValue),
					}
				}
			}
		}
	})
}

fn generate_create(
	variants: &[VariantOpts],
	input: &DeriveInput,
	kind: ChoiceKind,
) -> Result<TokenStream, TokenStream> {
	let option_type_variant = kind.option_type_variant();

	let dot_choices = {
		let mut dot_choices = Vec::<TokenStream>::new();

		let add_fn = kind.add_fn();

		for variant in variants {
			let name = variant.name();
			let value = variant.value(kind)?;

			let localizations: Vec<TokenStream> = variant
				.names
				.iter()
				.map(|(locale, string)| quote! { (#locale, #string) })
				.collect();

			let dot_choice = quote! {
				.#add_fn(
					#name,
					#value,
					[
						#(#localizations,)*
					]
				)
			};

			dot_choices.push(dot_choice);
		}

		dot_choices
	};

	Ok(quote! {
		fn create(name: impl Into<String>, desc: impl Into<String>) -> ::serenity::all::CreateCommandOption {
			::serenity::all::CreateCommandOption::new(
				::serenity::all::CommandOptionType::#option_type_variant
				name,
				desc,
			)
			#(#dot_choices)*
		}
	})
}
