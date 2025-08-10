use std::collections::HashMap;

use darling::{FromField, FromVariant, ast::Fields, util::Flag};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Type};

// TODO: Channel Types, Min/Max Value/Length
#[derive(Debug, Clone, FromField)]
#[darling(attributes(serein))]
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

	pub min_value: Option<syn::Expr>,
	pub max_value: Option<syn::Expr>,

	pub min_length: Option<u16>,
	pub max_length: Option<u16>,

	#[darling(default)]
	pub autocomplete: bool,
}

#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(serein))]
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

#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(serein))]
pub struct TopLevelOpts {
	pub ident: Ident,
	pub fields: Fields<VariantFieldOpts>,

	pub name: Option<String>,
	pub desc: String,

	#[darling(default)]
	pub names: HashMap<String, String>,

	#[darling(default)]
	pub descs: HashMap<String, String>,

	#[darling(default)]
	pub nsfw: bool,
}

impl From<TopLevelOpts> for VariantOpts {
	fn from(value: TopLevelOpts) -> Self {
		let TopLevelOpts {
			ident,
			fields,
			name,
			desc,
			names,
			descs,
			nsfw: _,
		} = value;
		Self {
			ident,
			fields,
			name,
			desc,
			names,
			descs,
		}
	}
}

#[derive(Debug, Clone, FromField)]
#[darling(attributes(serein))]
pub struct VariantFieldOpts {
	pub ty: Type,
}

#[derive(Debug, Clone, FromVariant)]
#[darling(attributes(serein))]
pub struct ChoiceOpts {
	pub ident: Ident,
	pub discriminant: Option<syn::Expr>,

	pub name: Option<String>,

	#[darling(default)]
	pub names: HashMap<String, String>,

	pub value: Option<syn::Expr>,
}

pub enum ChoiceKind {
	String,
	Int,
	Float,
}

pub fn touch_all_variants(input: &DeriveInput) -> TokenStream {
	let enum_name = &input.ident;
	match &input.data {
		syn::Data::Enum(data) => {
			let match_arms: Vec<TokenStream> = data
				.variants
				.iter()
				.map(|variant| {
					let variant_name = &variant.ident;

					quote! {
						Some(#enum_name::#variant_name(_)) => {}
					}
				})
				.collect();

			quote! {
				#[allow(unused)]
				const _: () = match None::<#enum_name> {
					#(#match_arms)*
					None => {}
				};
			}
		}
		_ => quote! {},
	}
}

pub fn derive_from_struct(
	fields: &[FieldOpts],
	input: &DeriveInput,
	cmd_trait: Ident,
) -> TokenStream {
	let self_fields: Vec<TokenStream> = fields
		.iter()
		.map(|field| {
			let FieldOpts {
				ident, ty, name, default, ..
			} = field;

			let ident = ident.as_ref().unwrap();
			let name = name.to_owned().unwrap_or_else(|| ident.to_string());

			let on_missing = if default.is_present() {
				quote! {
					<#ty as ::core::default::Default>::default()
				}
			} else {
				quote! {
					<#ty as ::serein::options::CommandOption>::try_from_missing_value()?
				}
			};

			quote! {
				#ident: match options.iter().filter(|opt| opt.name == #name).last() {
					Some(opt) => <#ty as ::serein::options::CommandOption>::try_from_resolved_value(opt.value.to_owned())?,
					None => #on_missing,
				}
			}
		})
		.collect();

	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		#[::serenity::async_trait]
		impl #impl_generics ::serein::slash::#cmd_trait for #ident #ty_generics #where_clause {
			async fn dispatch(
				ctx: ::serenity::all::Context,
				interaction: ::serenity::all::CommandInteraction,
			) -> ::serein::error::Result<()> {
				let options = interaction.data.options();

				let obj = Self {
					#(#self_fields,)*
				};

				serein::slash::Handler::handle(&obj, ctx, interaction).await
			}
		}
	}
}

pub fn derive_from_enum(
	variants: &[VariantOpts],
	input: &DeriveInput,
	cmd_trait: Ident,
	sub_cmd_trait: Ident,
) -> TokenStream {
	let match_arms: Vec<TokenStream> = variants
		.iter()
		.map(|variant| {
			let VariantOpts {
				ident,
				fields,
				name,
				..
			} = variant;

			let name = name.to_owned().unwrap_or_else(|| ident.to_string());
			let ty = &fields.fields.first().unwrap().ty;

			quote! {
				#name => <#ty as ::serein::slash::#sub_cmd_trait>::dispatch(ctx, interaction).await
			}
		})
		.collect();

	let ident = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	quote! {
		#[::serenity::async_trait]
		impl #impl_generics ::serein::slash::#cmd_trait for #ident #ty_generics #where_clause {
			async fn dispatch(
				ctx: ::serenity::all::Context,
				interaction: ::serenity::all::CommandInteraction,
			) -> ::serein::error::Result<()> {
				let options = interaction.data.options();

				let opt = match options.first() {
					Some(opt) => opt,
					None => return Err(serein::error::Error::UnrecognizedCommand)
				};

				let name = match opt.value {
					::serenity::all::ResolvedValue::SubCommand(_) => opt.name,
					::serenity::all::ResolvedValue::SubCommandGroup(_) => opt.name,
					_ => return Err(serein::error::Error::UnrecognizedCommand),
				};

				match name {
					#(#match_arms,)*
					_ => Err(serein::error::Error::UnrecognizedCommand),
				}
			}

		}
	}
}

pub fn make_option_creates(fields: &[FieldOpts]) -> Vec<TokenStream> {
	fields
		.iter()
		.map(|field| {
			let FieldOpts {
				ident,
				ty,
				name,
				desc,
				names,
				descs,
				default,
				min_value,
				max_value,
				min_length,
				max_length,
				autocomplete,
			} = field;

			let ident = ident.as_ref().unwrap();
			let name = name.to_owned().unwrap_or_else(|| ident.to_string());

			let dot_required = if default.is_present() {
				quote! { .required(false) }
			} else {
				quote! {}
			};

			let dot_min_length = match min_length {
				Some(x) => quote! { .min_length(#x) },
				None => quote! {},
			};
			let dot_max_length = match max_length {
				Some(x) => quote! { .max_length(#x) },
				None => quote! {},
			};

			let dot_names: Vec<TokenStream> = names
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! {
						.name_localized(#locale, #string)
					}
				})
				.collect();

			let dot_descs: Vec<TokenStream> = descs
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! {
						.description_localized(#locale, #string)
					}
				})
				.collect();

			quote! {
				<#ty as ::serein::options::CommandOption>::create(#name.to_owned(), #desc.to_owned())
					.set_autocomplete(#autocomplete)
					#dot_required
					#dot_min_length
					#dot_max_length
					#(#dot_names)*
					#(#dot_descs)*
			}
		})
		.collect()
}

pub fn make_sub_option_creates(variants: &[VariantOpts], sub_trait: Ident) -> Vec<TokenStream> {
	variants
		.iter()
		.map(|variant| {
			let VariantOpts {
				ident,
				fields,
				name,
				desc,
				names,
				descs,
			} = variant;

			let name = name.to_owned().unwrap_or_else(|| ident.to_string());
			let ty = &fields.fields.first().unwrap().ty;

			let dot_names: Vec<TokenStream> = names
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! {
						.name_localized(#locale, #string)
					}
				})
				.collect();

			let dot_descs: Vec<TokenStream> = descs
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! {
						.description_localized(#locale, #string)
					}
				})
				.collect();

			quote! {
				<#ty as ::serein::slash::#sub_trait>::create(#name)
					.description(#desc)
					#(#dot_names)*
					#(#dot_descs)*
			}
		})
		.collect()
}

pub fn make_top_level_creates(variants: &[TopLevelOpts], sub_trait: Ident) -> Vec<TokenStream> {
	variants
		.iter()
		.map(|variant| {
			let TopLevelOpts {
				ident,
				fields,
				name,
				desc,
				names,
				descs,
				nsfw,
			} = variant;

			let name = name.to_owned().unwrap_or_else(|| ident.to_string());
			let ty = &fields.fields.first().unwrap().ty;

			let dot_names: Vec<TokenStream> = names
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! {
						.name_localized(#locale, #string)
					}
				})
				.collect();

			let dot_descs: Vec<TokenStream> = descs
				.iter()
				.map(|(locale, string)| {
					let locale = locale.replace('_', "-");
					quote! {
						.description_localized(#locale, #string)
					}
				})
				.collect();

			quote! {
				<#ty as ::serein::slash::#sub_trait>::create(#name)
					.description(#desc)
					.nsfw(#nsfw)
					#(#dot_names)*
					#(#dot_descs)*
			}
		})
		.collect()
}
