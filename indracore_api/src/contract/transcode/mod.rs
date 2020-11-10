// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

mod decode;
mod encode;
mod scon;

use self::{
    // decode::decode_value,
    encode::encode_value,
    scon::Value,
    // scon::{Map, Value},
};

use anyhow::Result;
use ink_metadata::{ConstructorSpec, InkProject, MessageSpec};
// use scale::Input;
use scale_info::{
    form::{CompactForm, Form},
    Field, RegistryReadOnly, TypeDefComposite,
};
use std::fmt::{self, Debug, Display, Formatter};

/// Encode strings to SCALE encoded smart contract calls.
/// Decode SCALE encoded smart contract events and return values into `Value` objects.
pub struct Transcoder {
    metadata: InkProject,
}

impl Transcoder {
    pub fn new(metadata: InkProject) -> Self {
        Self { metadata }
    }

    fn registry(&self) -> &RegistryReadOnly {
        self.metadata.registry()
    }

    pub fn encode<I, S>(&self, name: &str, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str> + Debug,
    {
        let (selector, spec_args) = match (
            self.find_constructor_spec(name),
            self.find_message_spec(name),
        ) {
            (Some(c), None) => (c.selector(), c.args()),
            (None, Some(m)) => (m.selector(), m.args()),
            (Some(_), Some(_)) => {
                return Err(anyhow::anyhow!(
                    "Invalid metadata: both a constructor and message found with name '{}'",
                    name
                ))
            }
            (None, None) => {
                return Err(anyhow::anyhow!(
                    "No constructor or message with the name '{}' found",
                    name
                ))
            }
        };

        let mut encoded = selector.to_bytes().to_vec();
        for (spec, arg) in spec_args.iter().zip(args) {
            let value = arg.as_ref().parse::<scon::Value>()?;
            encode_value(self.registry(), spec.ty().ty().id(), &value, &mut encoded)?;
        }
        Ok(encoded)
    }

    fn constructors(&self) -> impl Iterator<Item = &ConstructorSpec<CompactForm>> {
        self.metadata.spec().constructors().iter()
    }

    fn messages(&self) -> impl Iterator<Item = &MessageSpec<CompactForm>> {
        self.metadata.spec().messages().iter()
    }

    fn find_message_spec(&self, name: &str) -> Option<&MessageSpec<CompactForm>> {
        self.messages()
            .find(|msg| msg.name().contains(&name.to_string()))
    }

    fn find_constructor_spec(&self, name: &str) -> Option<&ConstructorSpec<CompactForm>> {
        self.constructors()
            .find(|msg| msg.name().contains(&name.to_string()))
    }

    // pub fn decode_contract_event<I>(&self, data: &mut I) -> Result<ContractEvent>
    // where
    //     I: Input + Debug,
    // {
    //     let variant_index = data.read_byte()?;
    //     let event_spec = self
    //         .metadata
    //         .spec()
    //         .events()
    //         .get(variant_index as usize)
    //         .ok_or(anyhow::anyhow!(
    //             "Event variant {} not found in contract metadata",
    //             variant_index
    //         ))?;

    //     let mut args = Vec::new();
    //     for arg in event_spec.args() {
    //         let name = arg.name().to_string();
    //         let value = decode_value(self.registry(), arg.ty().ty().id(), data)?;
    //         args.push((Value::String(name), value));
    //     }

    //     let name = event_spec.name().to_string();
    //     let map = Map::new(Some(&name), args.into_iter().collect());

    //     Ok(ContractEvent {
    //         name,
    //         value: Value::Map(map),
    //     })
    // }

    // pub fn decode_return(&self, name: &str, data: Vec<u8>) -> Result<Value> {
    //     let msg_spec = self.find_message_spec(name).ok_or(anyhow::anyhow!(
    //         "Failed to find message spec with name '{}'",
    //         name
    //     ))?;
    //     if let Some(return_ty) = msg_spec.return_type().opt_type() {
    //         decode_value(self.registry(), return_ty.ty().id(), &mut &data[..])
    //     } else {
    //         Ok(Value::Unit)
    //     }
    // }
}

#[derive(Debug)]
pub enum CompositeTypeFields {
    StructNamedFields(Vec<CompositeTypeNamedField>),
    TupleStructUnnamedFields(Vec<Field<CompactForm>>),
    NoFields,
}

#[derive(Debug)]
pub struct CompositeTypeNamedField {
    name: <CompactForm as Form>::String,
    field: Field<CompactForm>,
}

impl CompositeTypeNamedField {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field(&self) -> &Field<CompactForm> {
        &self.field
    }
}

impl CompositeTypeFields {
    pub fn from_type_def(type_def: &TypeDefComposite<CompactForm>) -> Result<Self> {
        if type_def.fields().is_empty() {
            Ok(Self::NoFields)
        } else if type_def.fields().iter().all(|f| f.name().is_some()) {
            let fields = type_def
                .fields()
                .iter()
                .map(|f| CompositeTypeNamedField {
                    name: f.name().expect("All fields have a name; qed").to_owned(),
                    field: f.clone(),
                })
                .collect();
            Ok(Self::StructNamedFields(fields))
        } else if type_def.fields().iter().all(|f| f.name().is_none()) {
            Ok(Self::TupleStructUnnamedFields(type_def.fields().to_vec()))
        } else {
            Err(anyhow::anyhow!(
                "Struct fields should either be all named or all unnamed"
            ))
        }
    }
}

pub struct ContractEvent {
    pub name: String,
    pub value: Value,
}

impl Debug for ContractEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(&self, f)
    }
}

impl Display for ContractEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Value as Display>::fmt(&self.value, f)
    }
}
