use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// ###### Tree Errors ######

pub enum JecsTreeErrorType {
	WrongEntryType,
	IncompatibleOrMalformedData,
}

pub trait JecsTreeError : Error {
	fn error_type(&self) -> JecsTreeErrorType;
}

impl<T: JecsTreeError + 'static> From<T> for Box<dyn JecsTreeError> {
	fn from(value: T) -> Self {
		Box::new(value)
	}
}

// ### Wrong Entry Type ###

#[derive(Debug)]
pub struct JecsWrongEntryTypeError {
	pub expected_type: String,
	pub encountered_type: String,
}

impl Error for JecsWrongEntryTypeError {}

impl Display for JecsWrongEntryTypeError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Expected {} JECS data type, got {}", self.expected_type, self.encountered_type)?;
		Ok(())
	}
}

impl JecsTreeError for JecsWrongEntryTypeError {
	fn error_type(&self) -> JecsTreeErrorType {
		JecsTreeErrorType::WrongEntryType
	}
}

// ### Incompatible Or Malformed Data ###

#[derive(Debug)]
pub struct JecsIncompatibleOrMalformedError {
	pub data_type: String,
	pub value: String,
}

impl Error for JecsIncompatibleOrMalformedError {}

impl Display for JecsIncompatibleOrMalformedError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Failed to parse {} data with value '{}'", self.data_type, self.value)?;
		Ok(())
	}
}

impl JecsTreeError for JecsIncompatibleOrMalformedError {
	fn error_type(&self) -> JecsTreeErrorType {
		JecsTreeErrorType::IncompatibleOrMalformedData
	}
}

// ###### Parsing Errors ######

#[derive(Debug)]
pub struct JecsCorruptedDataError {
	pub row: usize,
	pub description: String,
}

impl Error for JecsCorruptedDataError {}

impl Display for JecsCorruptedDataError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "JECS file is corrupted. Line {}: {}", self.row, self.description)?;
		Ok(())
	}
}
