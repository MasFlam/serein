use std::collections::HashMap;

use darling::{FromField, FromMeta, FromVariant, ast::Fields, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

// TODO: Channel type, Autocomplete
#[derive(Debug, Clone, FromField)]
#[darling(attributes(serein), map = Self::after)]
pub struct FieldOpts {
	pub ident: Option<Ident>,
	pub ty: Type,

	pub name: Option<String>,
	pub desc: String,

	#[darling(default)]
	pub names: HashMap<String, String>,

	#[darling(default)]
	pub descs: HashMap<String, String>,

	pub default: Flag,

	pub min_value: Option<IntOrFloat>,
	pub max_value: Option<IntOrFloat>,

	pub min_length: Option<u16>,
	pub max_length: Option<u16>,

	pub autocomplete: Flag,
}

impl FieldOpts {
	fn after(mut self) -> Self {
		self.names = self
			.names
			.into_iter()
			.map(|(locale, string)| {
				let locale = locale.replace('_', "-");
				(locale, string)
			})
			.collect();

		self.descs = self
			.descs
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
			.unwrap_or_else(|| self.ident.as_ref().unwrap().to_string().to_lowercase())
	}
}

#[derive(Debug, Clone, FromMeta)]
pub enum IntOrFloat {
	Int(i64),
	Float(f64),
}

#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(serein), map = Self::after)]
pub struct VariantOpts {
	pub ident: Ident,
	pub fields: Fields<VariantFieldOpts>,

	pub name: Option<String>,
	pub desc: String,

	#[darling(default)]
	pub names: HashMap<String, String>,

	#[darling(default)]
	pub descs: HashMap<String, String>,
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

		self.descs = self
			.descs
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

	pub fn ty(&self) -> &Type {
		&self.fields.fields[0].ty
	}
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(serein))]
pub struct VariantFieldOpts {
	pub ty: Type,
}

pub fn generate_opt_creates(fields: &[FieldOpts]) -> Vec<TokenStream> {
	let mut sub_opt_creates = Vec::<TokenStream>::new();

	for field in fields {
		let name = field.name();
		let ty = &field.ty;
		let desc = &field.desc;

		let dot_required = if field.default.is_present() {
			quote! { .required(false) }
		} else {
			quote! {}
		};

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

		let dot_min_value = match field.min_value {
			Some(IntOrFloat::Int(value)) => quote! { .min_int_value(#value) },
			Some(IntOrFloat::Float(value)) => quote! { .min_number_value(#value) },
			None => quote! {},
		};

		let dot_max_value = match field.max_value {
			Some(IntOrFloat::Int(value)) => quote! { .max_int_value(#value) },
			Some(IntOrFloat::Float(value)) => quote! { .max_number_value(#value) },
			None => quote! {},
		};

		let dot_min_length = match field.min_length {
			Some(value) => quote! { .min_length(#value) },
			None => quote! {},
		};

		let dot_max_length = match field.max_length {
			Some(value) => quote! { .max_length(#value) },
			None => quote! {},
		};

		let create = quote! {
			<#ty as ::serein::options::CommandOption>::create(#name, #desc)
				#(#dot_names)*
				#(#dot_descs)*
				#dot_required
				#dot_min_value
				#dot_max_value
				#dot_min_length
				#dot_max_length
		};

		sub_opt_creates.push(create);
	}

	sub_opt_creates
}

pub fn generate_sub_or_subsub_create_from_struct(fields: &[FieldOpts]) -> TokenStream {
	let sub_opt_creates = generate_opt_creates(fields);

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
