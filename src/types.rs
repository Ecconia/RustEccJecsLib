use std::collections::HashMap;
use crate::errors::{JecsIncompatibleOrMalformedError, JecsTreeError, JecsWrongEntryTypeError};

#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub enum JecsType {
	Any(), //Could be literally any of the below types, but always a length of zero
	Value(String), //Contains a single text value
	Map(HashMap<String, JecsType>), //Contains a dictionary
	List(Vec<JecsType>), //Contains a list
}

//Functions to check the JECS entry type and
impl JecsType {
	pub fn name(&self) -> &str {
		match self {
			JecsType::Any{..} => "Any",
			JecsType::Value{..} => "Value",
			JecsType::Map{..} => "Map",
			JecsType::List{..} => "List",
		}
	}
	
	pub fn is_any(&self) -> bool {
		match self {
			JecsType::Any{..} => true,
			_ => false,
		}
	}
	
	pub fn is_value(&self) -> bool {
		match self {
			JecsType::Value{..} => true,
			_ => false,
		}
	}
	
	pub fn get_value(&self) -> Option<&str> {
		if let JecsType::Value(value) = self {
			return Some(value);
		}
		return None;
	}
	
	pub fn is_map(&self) -> bool {
		match self {
			JecsType::Any{..} => true,
			JecsType::Map{..} => true,
			_ => false,
		}
	}
	
	pub fn get_map(&self) -> Option<&HashMap<String, JecsType>> {
		if let JecsType::Map(value) = self {
			return Some(value);
		}
		return None;
	}
	
	pub fn is_list(&self) -> bool {
		match self {
			JecsType::Any{..} => true,
			JecsType::List{..} => true,
			_ => false,
		}
	}
	
	pub fn get_list(&self) -> Option<&Vec<JecsType>> {
		if let JecsType::List(value) = self {
			return Some(value);
		}
		return None;
	}
}

impl JecsType {
	pub fn expect_map(&self) -> Result<&HashMap<String, JecsType>, JecsWrongEntryTypeError> {
		if !self.is_map() {
			return Err(JecsWrongEntryTypeError {
				expected_type: "MAP".to_string(),
				encountered_type: self.name().to_string(),
			});
		}
		Ok(self.get_map().unwrap())
	}
	
	pub fn expect_list(&self) -> Result<&Vec<JecsType>, JecsWrongEntryTypeError> {
		if !self.is_list() {
			return Err(JecsWrongEntryTypeError {
				expected_type: "LIST".to_string(),
				encountered_type: self.name().to_string(),
			});
		}
		Ok(self.get_list().unwrap())
	}
	
	pub fn expect_string(&self) -> Result<&str, JecsWrongEntryTypeError> {
		if !self.is_value() {
			return Err(JecsWrongEntryTypeError {
				expected_type: "VALUE".to_string(),
				encountered_type: self.name().to_string(),
			});
		}
		Ok(self.get_value().unwrap())
	}
	
	pub fn expect_bool(&self) -> Result<bool, Box<dyn JecsTreeError>> {
		let value = self.expect_string().map_err(|mut e| { e.expected_type = "bool".to_string(); e })?;
		Ok(match value {
			"true" | "on" | "yes" | "y" => true,
			"false" | "off" | "no" | "n" => false,
			_ => {
				Err(JecsIncompatibleOrMalformedError {
					data_type: "boolean".to_string(),
					value: value.to_string(),
				})?
			}
		})
	}
	
	pub fn expect_double(&self) -> Result<f64, Box<dyn JecsTreeError>> {
		let value = self.expect_string().map_err(|mut e| { e.expected_type = "double".to_string(); e })?;
		Ok(value.parse::<f64>().map_err(|_| JecsIncompatibleOrMalformedError {
			data_type: "double".to_string(),
			value: value.to_string(),
		})?)
	}
	
	pub fn expect_color(&self) -> Result<(u8, u8, u8), Box<dyn JecsTreeError>> {
		let value = self.expect_string().map_err(|mut e| { e.expected_type = "color".to_string(); e })?;
		if value.len() != 6 {
			//Not 6 characters long...
			Err(JecsIncompatibleOrMalformedError {
				data_type: "color".to_string(),
				value: value.to_string(),
			})?;
		}
		if value.chars().position(|c| {
			!(c >= '0' && c <= '9' || c >= 'A' && c <= 'F')
		}).is_some() {
			//Wrong characters, allowed: [0-9A-F]
			Err(JecsIncompatibleOrMalformedError {
				data_type: "color".to_string(),
				value: value.to_string(),
			})?;
		}
		//Data validated, time to parse it:
		Ok((
			u8::from_str_radix(&value[0..2], 16).unwrap(),
			u8::from_str_radix(&value[2..4], 16).unwrap(),
			u8::from_str_radix(&value[4..6], 16).unwrap(),
		))
	}
	
	pub fn expect_unsigned(&self) -> Result<u32, Box<dyn JecsTreeError>> {
		let value = self.expect_string().map_err(|mut e| { e.expected_type = "unsigned".to_string(); e })?;
		Ok(value.parse::<u32>().map_err(|_e| JecsIncompatibleOrMalformedError {
			data_type: "unsigned".to_string(),
			value: value.to_string(),
		})?)
	}
	
	pub fn expect_component_address(&self) -> Result<u32, Box<dyn JecsTreeError>> {
		let mut value = self.expect_string().map_err(|mut e| { e.expected_type = "component address".to_string(); e })?;
		if !value.starts_with("C-") {
			//Must start with 'C-'
			Err(JecsIncompatibleOrMalformedError {
				data_type: "component address".to_string(),
				value: value.to_string(),
			})?;
		}
		value = &value[2..];
		Ok(value.parse::<u32>().map_err(|_| JecsIncompatibleOrMalformedError {
			data_type: "component address".to_string(),
			value: value.to_string(),
		})?)
	}
}
