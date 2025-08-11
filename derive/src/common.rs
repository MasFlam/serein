use std::collections::HashMap;

use darling::{FromField, FromMeta, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

// TODO: Channel Types, Min/Max Value/Length
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

	pub fn min_value(&self) -> Option<TokenStream> {
		self.min_value.as_ref().map(|value| match value {
			IntOrFloat::Int(value) => quote!(#value),
			IntOrFloat::Float(value) => quote!(#value),
		})
	}

	pub fn max_value(&self) -> Option<TokenStream> {
		self.max_value.as_ref().map(|value| match value {
			IntOrFloat::Int(value) => quote!(#value),
			IntOrFloat::Float(value) => quote!(#value),
		})
	}
}

#[derive(Debug, Clone, FromMeta)]
pub enum IntOrFloat {
	Int(i64),
	Float(f64),
}
