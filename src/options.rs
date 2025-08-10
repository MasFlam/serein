use serenity::all::{
	Attachment, AttachmentId, ChannelId, CommandOptionType, CreateCommandOption, GenericId,
	PartialChannel, PartialMember, ResolvedValue, Role, RoleId, Unresolved, User, UserId,
};

use crate::error::{Error, Result};

pub use serein_derive::{FloatChoice, IntChoice, StringChoice};

pub trait CommandOption: Sized {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self>;

	fn try_from_missing_value() -> Result<Self> {
		Err(Error::MissingOption)
	}

	fn create(name: String, desc: String) -> CreateCommandOption;
}

macro_rules! impl_create {
	($kind:expr) => {
		fn create(name: String, desc: String) -> CreateCommandOption {
			CreateCommandOption::new($kind, name, desc).required(true)
		}
	};
}

impl CommandOption for String {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::String(value) => Ok(value.to_owned()),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::String);
}

macro_rules! from_resolved_value_impl_integer {
	($t:ty) => {
		impl CommandOption for $t {
			fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
				match value {
					ResolvedValue::Integer(value) => {
						value.try_into().map_err(|_| Error::BadOptionValue)
					}
					_ => Err(Error::BadOptionType),
				}
			}

			impl_create!(CommandOptionType::Integer);
		}
	};
}

from_resolved_value_impl_integer!(isize);
from_resolved_value_impl_integer!(i128);
from_resolved_value_impl_integer!(i64);
from_resolved_value_impl_integer!(i32);
from_resolved_value_impl_integer!(i16);
from_resolved_value_impl_integer!(i8);
from_resolved_value_impl_integer!(usize);
from_resolved_value_impl_integer!(u128);
from_resolved_value_impl_integer!(u64);
from_resolved_value_impl_integer!(u32);
from_resolved_value_impl_integer!(u16);
from_resolved_value_impl_integer!(u8);

impl CommandOption for bool {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::Boolean(value) => Ok(value),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Boolean);
}

impl CommandOption for User {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::User(user, _) => Ok(user.to_owned()),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::User);
}

impl CommandOption for PartialMember {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::User(_, partial_member) => match partial_member {
				Some(partial_member) => Ok(partial_member.to_owned()),
				None => Err(Error::BadOptionValue),
			},
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::User);
}

impl CommandOption for (User, PartialMember) {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::User(user, partial_member) => match partial_member {
				Some(partial_member) => Ok((user.to_owned(), partial_member.to_owned())),
				None => Err(Error::BadOptionValue),
			},
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::User);
}

impl CommandOption for PartialChannel {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::Channel(value) => Ok(value.to_owned()),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Channel);
}

impl CommandOption for Role {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::Role(value) => Ok(value.to_owned()),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Role);
}

macro_rules! from_resolved_value_impl_number {
	($t:ty) => {
		impl CommandOption for $t {
			fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
				match value {
					ResolvedValue::Number(value) => Ok(value as Self),
					_ => Err(Error::BadOptionType),
				}
			}

			impl_create!(CommandOptionType::Number);
		}
	};
}

from_resolved_value_impl_number!(f64);
from_resolved_value_impl_number!(f32);

impl CommandOption for Attachment {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::Attachment(value) => Ok(value.to_owned()),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Attachment);
}

impl CommandOption for UserId {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::User(user, _) => Ok(user.id),
			ResolvedValue::Unresolved(Unresolved::User(id)) => Ok(id),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::User);
}

impl CommandOption for RoleId {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::Role(role) => Ok(role.id),
			ResolvedValue::Unresolved(Unresolved::RoleId(id)) => Ok(id),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Role);
}

impl CommandOption for ChannelId {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::Channel(channel) => Ok(channel.id),
			ResolvedValue::Unresolved(Unresolved::Channel(id)) => Ok(id),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Channel);
}

impl CommandOption for GenericId {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::User(user, _) => Ok(Self::new(user.id.get())),
			ResolvedValue::Role(role) => Ok(Self::new(role.id.get())),
			ResolvedValue::Unresolved(Unresolved::Mentionable(id)) => Ok(id),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Mentionable);
}

impl CommandOption for AttachmentId {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		match value {
			ResolvedValue::Unresolved(Unresolved::Attachment(id)) => Ok(id),
			_ => Err(Error::BadOptionType),
		}
	}

	impl_create!(CommandOptionType::Attachment);
}

impl<T: CommandOption> CommandOption for Option<T> {
	fn try_from_resolved_value(value: ResolvedValue) -> Result<Self> {
		T::try_from_resolved_value(value).map(Some)
	}

	fn try_from_missing_value() -> Result<Self> {
		Ok(None)
	}

	fn create(name: String, desc: String) -> CreateCommandOption {
		<T as CommandOption>::create(name, desc).required(false)
	}
}

pub trait StringChoice: Sized {
	fn from_value(value: &str) -> Result<Self>;
	fn create_with_choices(name: String, desc: String) -> CreateCommandOption;
}

pub trait IntChoice: Sized {
	fn from_value(value: i64) -> Result<Self>;
	fn create_with_choices(name: String, desc: String) -> CreateCommandOption;
}

pub trait FloatChoice: Sized {
	fn from_value(value: f64) -> Result<Self>;
	fn create_with_choices(name: String, desc: String) -> CreateCommandOption;
}
