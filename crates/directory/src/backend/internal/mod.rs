/*
 * Copyright (c) 2023 Stalwart Labs Ltd.
 *
 * This file is part of Stalwart Mail Server.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of
 * the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 * in the LICENSE file at the top-level directory of this distribution.
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * You can be released from the requirements of the AGPLv3 license by
 * purchasing a commercial license. Please contact licensing@stalw.art
 * for more details.
*/

pub mod lookup;
pub mod manage;

use std::slice::Iter;

use store::{write::key::KeySerializer, Deserialize, Serialize, U32_LEN};
use utils::codec::leb128::Leb128Iterator;

use crate::{Principal, Type};

impl Serialize for Principal<u32> {
    fn serialize(self) -> Vec<u8> {
        (&self).serialize()
    }
}

impl Serialize for &Principal<u32> {
    fn serialize(self) -> Vec<u8> {
        let mut serializer = KeySerializer::new(
            U32_LEN * 3
                + 2
                + self.name.len()
                + self.emails.iter().map(|s| s.len()).sum::<usize>()
                + self.secrets.iter().map(|s| s.len()).sum::<usize>()
                + self.member_of.len() * U32_LEN
                + self.description.as_ref().map(|s| s.len()).unwrap_or(0),
        )
        .write(1u8)
        .write_leb128(self.id)
        .write(self.typ as u8)
        .write_leb128(self.quota)
        .write_leb128(self.name.len())
        .write(self.name.as_bytes())
        .write_leb128(self.description.as_ref().map_or(0, |s| s.len()))
        .write(self.description.as_deref().unwrap_or_default().as_bytes());

        for list in [&self.secrets, &self.emails] {
            serializer = serializer.write_leb128(list.len());
            for value in list {
                serializer = serializer.write_leb128(value.len()).write(value.as_bytes());
            }
        }

        serializer = serializer.write_leb128(self.member_of.len());
        for id in &self.member_of {
            serializer = serializer.write_leb128(*id);
        }

        serializer.finalize()
    }
}

impl Deserialize for Principal<u32> {
    fn deserialize(bytes: &[u8]) -> store::Result<Self> {
        deserialize(bytes)
            .ok_or_else(|| store::Error::InternalError("Failed to deserialize principal".into()))
    }
}

fn deserialize(bytes: &[u8]) -> Option<Principal<u32>> {
    let mut bytes = bytes.iter();
    if bytes.next()? != &1 {
        return None;
    }

    Principal {
        id: bytes.next_leb128()?,
        typ: Type::from_u8(*bytes.next()?),
        quota: bytes.next_leb128()?,
        name: deserialize_string(&mut bytes)?,
        description: deserialize_string(&mut bytes).map(|v| {
            if !v.is_empty() {
                Some(v)
            } else {
                None
            }
        })?,
        secrets: deserialize_string_list(&mut bytes)?,
        emails: deserialize_string_list(&mut bytes)?,
        member_of: deserialize_u32_list(&mut bytes)?,
    }
    .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrincipalField {
    #[serde(rename = "name")]
    Name,
    #[serde(rename = "type")]
    Type,
    #[serde(rename = "quota")]
    Quota,
    #[serde(rename = "description")]
    Description,
    #[serde(rename = "secrets")]
    Secrets,
    #[serde(rename = "emails")]
    Emails,
    #[serde(rename = "memberOf")]
    MemberOf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrincipalUpdate {
    action: PrincipalAction,
    field: PrincipalField,
    value: PrincipalValue,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrincipalAction {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "addItem")]
    AddItem,
    #[serde(rename = "removeItem")]
    RemoveItem,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum PrincipalValue {
    String(String),
    StringList(Vec<String>),
    Integer(u32),
    Type(Type),
}

impl PrincipalUpdate {
    pub fn set(field: PrincipalField, value: PrincipalValue) -> PrincipalUpdate {
        PrincipalUpdate {
            action: PrincipalAction::Set,
            field,
            value,
        }
    }

    pub fn add_item(field: PrincipalField, value: PrincipalValue) -> PrincipalUpdate {
        PrincipalUpdate {
            action: PrincipalAction::AddItem,
            field,
            value,
        }
    }

    pub fn remove_item(field: PrincipalField, value: PrincipalValue) -> PrincipalUpdate {
        PrincipalUpdate {
            action: PrincipalAction::RemoveItem,
            field,
            value,
        }
    }
}

fn deserialize_string(bytes: &mut Iter<'_, u8>) -> Option<String> {
    let len = bytes.next_leb128()?;
    let mut string = Vec::with_capacity(len);
    for _ in 0..len {
        string.push(*bytes.next()?);
    }
    String::from_utf8(string).ok()
}

fn deserialize_string_list(bytes: &mut Iter<'_, u8>) -> Option<Vec<String>> {
    let len = bytes.next_leb128()?;
    let mut list = Vec::with_capacity(len);
    for _ in 0..len {
        list.push(deserialize_string(bytes)?);
    }
    Some(list)
}

fn deserialize_u32_list(bytes: &mut Iter<'_, u8>) -> Option<Vec<u32>> {
    let len = bytes.next_leb128()?;
    let mut list = Vec::with_capacity(len);
    for _ in 0..len {
        list.push(bytes.next_leb128()?);
    }
    Some(list)
}

impl Type {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Type::Individual,
            1 => Type::Group,
            2 => Type::Resource,
            3 => Type::Location,
            4 => Type::Superuser,
            5 => Type::List,
            _ => Type::Other,
        }
    }
}