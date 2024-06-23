use std::{collections::HashMap, fs, str::from_utf8};
use std::cmp::{Ordering, PartialEq};
use std::error::Error;
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

use crate::errors::JecsCorruptedDataError;
use crate::types::JecsType;

pub fn parse_jecs_file(path: &Path) -> Result<JecsType, Box<dyn Error>> {
	let bytes = fs::read(&path)?; //std::io::Error
	parse_jecs_bytes(&bytes)
}

pub fn parse_jecs_bytes(bytes: &[u8]) -> Result<JecsType, Box<dyn Error>> {
	let text = from_utf8(bytes)?; //Utf8Error
	Ok(parse_jecs_string(text)?)
}

pub fn parse_jecs_string(text: &str) -> Result<JecsType, JecsCorruptedDataError> {
	let mut tree_parser = TreeParser::default();
	
	let mut line_iterator = text.lines()
		.into_iter()
		.enumerate().map(|(index, line)| (index + 1, line))
		.peekable();
	//The stack is still empty, handle the very first line (differently):
	while let Some(line_data) = line_iterator.next() {
		if let Some(line_meta) = parse_line(line_data, &mut line_iterator)? {
			tree_parser.add_validate_root(line_meta)?;
			break;
		}
	}
	//Process every remaining line of the file:
	while let Some(line_data) = line_iterator.next() {
		if let Some(line_meta) = parse_line(line_data, &mut line_iterator)? {
			tree_parser.append_next_line(line_meta)?;
		}
	}
	//Empty the stack, so that only root elements and their child structures remain:
	tree_parser.post_line_addition_cleanup();
	
	//Finally convert everything to JECS type structures without the meta & temporary information:
	Ok(tree_parser.finalize_to_map()?)
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
enum JecsTypeInner {
	Any,
	Value,
	Map,
	List,
}

#[derive(Debug)]
struct LineMeta {
	row: usize,
	indentation: usize,
	key: Option<String>,
	value: Option<String>,
}

impl LineMeta {
	fn is_list(&self) -> bool {
		self.key.is_none()
	}
	
	fn get_data_type(&self) -> JecsTypeInner {
		if self.is_list() {
			JecsTypeInner::List
		} else {
			JecsTypeInner::Map
		}
	}
	
	fn is_parent(&self) -> bool {
		self.value.is_none()
	}
}

macro_rules! jecs_error {
	($row:expr, $($arguments:tt)*) => {
		Err(JecsCorruptedDataError {
			row: $row,
			description: format!($($arguments)*),
		})?
	};
}

fn parse_line<'a>((row, line): (usize, &str), line_iterator: &mut Peekable<impl Iterator<Item = (usize, &'a str)>>) -> Result<Option<LineMeta>, JecsCorruptedDataError> {
	let mut iterator = line.chars().peekable();
	
	//Read indentation:
	let indentation = match read_indentation(row, &mut iterator, true)? {
		None => return Ok(None),
		Some(indentation) => indentation,
	};
	//At this point, we know that there still is a symbol, as we used 'break'.
	
	//Read key:
	let key = read_key(row, &mut iterator)?;
	
	//Skip space until value:
	while iterator.peek().is_some() && *iterator.peek().unwrap() == ' ' {
		iterator.next();
	}
	
	//Read value:
	let value = read_value(row, indentation, &mut iterator, line_iterator)?;
	
	return Ok(Some(LineMeta {
		row,
		indentation,
		key,
		value,
	}));
	
	fn read_indentation(row: usize, iterator: &mut Peekable<Chars>, check_for_column: bool) -> Result<Option<usize>, JecsCorruptedDataError> {
		let mut indentation = 0;
		loop {
			let c = match iterator.peek() {
				None => return Ok(None), //Empty line
				Some(c) => *c,
			};
			
			if c == ' ' {
				indentation += 1;
				iterator.next(); //Consume the character from the line
			} else if c == '#' {
				return Ok(None); //This line only contains a comment.
			} else if check_for_column && c == ':' {
				jecs_error!(row, "Line has no key, encountered ':'");
			} else {
				//Whatever character comes here, it must be part of the key. Do not consume.
				break;
			}
		}
		Ok(Some(indentation))
	}
	
	fn read_key(row: usize, iterator: &mut Peekable<Chars>) -> Result<Option<String>, JecsCorruptedDataError> {
		if *iterator.peek().unwrap() != '-' {
			let mut key_builder = String::new();
			loop {
				let c = match iterator.next() {
					None => {
						jecs_error!(row, "Unexpected line end while reading key") //Key never completely read
					}
					Some(c) => c,
				};
				
				if c == ':' {
					//Encountered the end of the key. Stop the loop, but consume the column (its part of the key).
					break;
				} else if c == '#' {
					jecs_error!(row, "key may not contain a # character"); //Key never completely read
				} else {
					key_builder.push(c);
				}
			}
			//Remove any trailing spaces from the key. As a key may not have spaces at its end.
			Ok(Some(key_builder.trim_end_matches(|c| c == ' ').to_string()))
		} else {
			iterator.next(); //Skip the '-', as it is part of the key.
			Ok(None) //This is a "list entry", thus there is no key.
		}
	}
	
	fn read_value<'a>(mut row: usize, original_indentation: usize, iterator: &mut Peekable<Chars>, line_iterator: &mut Peekable<impl Iterator<Item = (usize, &'a str)>>) -> Result<Option<String>, JecsCorruptedDataError> {
		let content = read_value_raw(iterator);
		if content.is_none() || content.as_ref().unwrap() != "\"\"\"" {
			//Not a multi-line string, return
			return Ok(content);
		}
		//Value is a multi-line string, thus read more lines until the value is fully read:
		let mut string_builder = String::new();
		let mut last_indentation = None;
		let mut wrote_first_line = false; //Newlines are added as separator before new content. This flag keeps track if there is content to separate or not.
		loop {
			//Get next line:
			let tuple = line_iterator.next();
			if tuple.is_none() {
				jecs_error!(row, "Multi-line string started, but file ends unexpectedly");
			}
			let (next_row, content) = tuple.unwrap();
			row = next_row; //Update the row index, to show correct row in errors
			let mut iterator = content.chars().peekable();
			
			//Get indentation (and skip spaces) of next line:
			let indentation = match read_indentation(row, &mut iterator, false)? {
				None => {
					//Line simply ends, save a newline and proceed with the next line
					if wrote_first_line {
						string_builder.push('\n');
					}
					wrote_first_line = true;
					continue;
				}
				Some(indentation) => indentation,
			};
		
			//Handle indentation, validate the string line indentations:
			match last_indentation {
				None => {
					//First line, check and save indentation
					if indentation <= original_indentation {
						jecs_error!(row, "Multi-line string lines must have more indentation than its opener");
					}
					last_indentation = Some(indentation);
				}
				Some(last_indentation) => {
					if last_indentation != indentation {
						jecs_error!(row, "Multi-line string lines must have consistent indentation until its terminator (\"\"\")");
					}
				}
			}
		
			//Get actual content:
			let content = read_value_raw(&mut iterator).unwrap(); //It is impossible to get None here, as the indentation check would have terminated then.
			if content == "\"\"\"" {
				//Found termination of multi-line string.
				return Ok(Some(string_builder));
			}
			if wrote_first_line {
				string_builder.push('\n');
			}
			string_builder.push_str(&content);
			wrote_first_line = true;
		}
	}
	
	fn read_value_raw(iterator: &mut Peekable<Chars>) -> Option<String> {
		if iterator.peek().is_none() || *iterator.peek().unwrap() == '#' {
			None //The line has no value as it reached the end. Or the line has reached a comment and thus there is no value.
		} else {
			let mut value_builder = String::new();
			//It is ensured, that the very first character exists and is not a comment.
			loop {
				let c = match iterator.next() {
					None => break,
					Some(c) => c,
				};
				
				if c == '\\' && iterator.peek().is_some() && *iterator.peek().unwrap() == '#' {
					value_builder.push('#');
					iterator.next(); //Skip the '#'
				} else if c == '#' {
					break; //Reached end of content, rest is comment.
				} else {
					//Append normal data:
					value_builder.push(c);
				}
			}
			
			Some(value_builder.trim_end_matches(|c| c == ' ').to_string())
		}
	}
}

#[derive(Debug)]
struct LineContext {
	meta: LineMeta,
	children: Vec<LineContext>,
	expected_child_indentation: usize,
	determined_type: JecsTypeInner,
}

impl LineContext {
	fn new(meta: LineMeta) -> Self {
		Self {
			determined_type: if meta.is_parent() { JecsTypeInner::Any } else { JecsTypeInner::Value },
			meta,
			children: Vec::new(),
			expected_child_indentation: 0,
		}
	}
}

struct TreeParser {
	roots: Vec<LineContext>,
	stack: Vec<LineContext>,
}

impl Default for TreeParser {
	fn default() -> Self {
		Self {
			roots: Vec::new(),
			stack: Vec::new(),
		}
	}
}

impl TreeParser {
	fn add_validate_root(&mut self, line_meta: LineMeta) -> Result<(), JecsCorruptedDataError> {
		if line_meta.indentation != 0 {
			jecs_error!(line_meta.row, "Root level entries need indentation level {}, but got {}", 0, line_meta.indentation);
		}
		if line_meta.is_list() {
			jecs_error!(line_meta.row, "Root level entries need a key, they may not be list entries");
		}
		self.stack.push(LineContext::new(line_meta));
		Ok(())
	}
	
	fn append_next_line(&mut self, current_line_meta: LineMeta) -> Result<(), JecsCorruptedDataError> {
		let previous_line = self.stack.last_mut().unwrap();
		match current_line_meta.indentation.cmp(&previous_line.meta.indentation) {
			Ordering::Greater => {
				//New child entry.
				self.handle_new_child_line(current_line_meta)?;
			}
			Ordering::Equal => {
				//Sibling entry.
				self.handle_new_sibling_line(current_line_meta)?;
			}
			Ordering::Less => {
				//Some parent's sibling entry.
				self.handle_new_parents_sibling_line(current_line_meta)?;
			}
		}
		
		return Ok(());
	}
	
	fn handle_new_child_line(&mut self, current_line_meta: LineMeta) -> Result<(), JecsCorruptedDataError> {
		let previous_line = self.stack.last_mut().unwrap(); //For borrowing reasons, this has to be queried here again.
		//Parent node type MUST be Any (no value):
		if previous_line.determined_type != JecsTypeInner::Any {
			jecs_error!(current_line_meta.row, "Child entries can only be added to entries without value");
		}
		//Indentation and type of the parent entry, can only be inferred from the child entry. Apply now:
		previous_line.determined_type = if current_line_meta.is_list() { JecsTypeInner::List } else { JecsTypeInner::Map };
		previous_line.expected_child_indentation = current_line_meta.indentation;
		
		self.stack.push(LineContext::new(current_line_meta));
		Ok(())
	}
	
	fn handle_new_sibling_line(&mut self, current_line_meta: LineMeta) -> Result<(), JecsCorruptedDataError> {
		//First remove the previous entry, and inject it into the previous parent (or root):
		let previous_line = self.stack.pop().unwrap();
		if self.stack.is_empty() {
			//Save the old root node and replace with a new one:
			self.roots.push(previous_line);
			self.add_validate_root(current_line_meta)?; //The indentation validation here is not required.
		} else {
			//We got a parent node. Merge previous into that and take its place.
			let parent = self.stack.last_mut().unwrap();
			if parent.determined_type != current_line_meta.get_data_type() {
				jecs_error!(current_line_meta.row, "Cannot mix list and dict collection entries with the same parent");
			}
			parent.children.push(previous_line);
			//Take the place of the previous line
			self.stack.push(LineContext::new(current_line_meta));
		}
		Ok(())
	}
	
	fn handle_new_parents_sibling_line(&mut self, current_line_meta: LineMeta) -> Result<(), JecsCorruptedDataError> {
		loop {
			//There exists an element with higher indentation, thus it has to be removed and merged to its parent.
			//This may have to be done repeatedly as long as there is an entry on the stack with higher indentation.
			let previous_entry_with_higher_indentation = self.stack.pop().unwrap();
			if self.stack.is_empty() {
				//Stack is empty, we must be adding a new root level entry.
				//Save the old root node and replace with a new one:
				self.roots.push(previous_entry_with_higher_indentation);
				self.add_validate_root(current_line_meta)?;
				break; //Done, as the new entry is injected properly.
			} else {
				//The stack is not empty, thus add the top line to its parent on the stack:
				let potential_parent = self.stack.last_mut().unwrap();
				potential_parent.children.push(previous_entry_with_higher_indentation);
				//From now on, work with the parent (now top from stack).
				
				//First confirm, that the indentation is not above the next parent. As that would be impossible.
				//We have less indentation for this line that the child of the parent, thus the indentation cannot be bigger than the parents child indentation.
				if current_line_meta.indentation > potential_parent.expected_child_indentation {
					jecs_error!(current_line_meta.row, "Wrongly indented JECS entry! Expected indentation {} but got {}", potential_parent.expected_child_indentation, current_line_meta.indentation);
				}
				//Check if the indentation level is the same as the current parent. If that is the case, we found the correct new parent.
				if current_line_meta.indentation == potential_parent.expected_child_indentation {
					if potential_parent.determined_type != current_line_meta.get_data_type() {
						jecs_error!(current_line_meta.row, "Cannot mix list and dict collection entries within the same parent");
					}
					
					self.stack.push(LineContext::new(current_line_meta));
					break;
				}
				//else The indentation level is below the parent, thus the current line must be the child of another parent (or root).
				//Repeat the process with the next parent.
			}
		}
		Ok(())
	}
	
	fn post_line_addition_cleanup(&mut self) {
		//Merge every stack entry into its parent, until the stack is empty.
		//The last stack entry gets added as a root entry.
		while !self.stack.is_empty() {
			let entry = self.stack.pop().unwrap();
			if let Some(parent) = self.stack.last_mut() {
				parent.children.push(entry);
			} else {
				self.roots.push(entry);
			}
		}
	}
	
	fn finalize_to_map(self) -> Result<JecsType, JecsCorruptedDataError> {
		struct ConvertedMeta {
			name: Option<String>,
			converted: JecsType,
			child_count: usize,
		}
		//Create a root component, which the map can be extracted from later:
		let mut converted_stack = vec![ConvertedMeta {
			name: None,
			converted: JecsType::Map(HashMap::with_capacity(self.roots.len())),
			child_count: self.roots.len(),
		}];
		let mut process_stack : Vec<LineContext> = self.roots.into_iter().rev().collect();
		
		while let Some(mut entry) = process_stack.pop() {
			//First create a converted Jecs type without child components:
			let converted_entry = match entry.determined_type {
				JecsTypeInner::Any => JecsType::Any(),
				JecsTypeInner::Value => {
					JecsType::Value(entry.meta.value.take().unwrap())
				},
				JecsTypeInner::Map => {
					JecsType::Map(HashMap::with_capacity(entry.children.len()))
				}
				JecsTypeInner::List => {
					JecsType::List(Vec::with_capacity(entry.children.len()))
				}
			};
			
			if entry.children.len() == 0 {
				//If the entry has no children, it needs to immediately be injected into its parent (on the converted stack).
				//If that or further parent has all of their children, they get merged into their parent as well.
				
				//Keep a reference to the latest parent and create a new converted meta entry for the currently converted entry:
				let mut parent = converted_stack.last_mut().unwrap();
				let mut child = Some(ConvertedMeta {
					name: entry.meta.key.take(),
					converted: converted_entry,
					child_count: entry.children.len()
				});
				//The process for all iterations stays the same, only parent and child variables need to be updated.
				loop {
					//Add the child into the parent component. During that, check if the parent is full (has_more).
					let mut has_more = false;
					if let JecsType::Map(ref mut map) = &mut parent.converted {
						let converted_meta = child.take().unwrap();
						map.insert(converted_meta.name.unwrap(), converted_meta.converted);
						has_more = parent.child_count > map.len();
					} else if let JecsType::List(ref mut list) = &mut parent.converted {
						list.push(child.take().unwrap().converted);
						has_more = parent.child_count > list.len();
					} //else - impossible.
					
					if has_more || converted_stack.len() <= 1 {
						//Parent is not full, or there is no more child to merge on the converted stack.
						break; //Stop and inject the next entry.
					}
					//Else there is no more child to add to the parent, thus merge that parent as well:
					child = Some(converted_stack.pop().unwrap());
					parent = converted_stack.last_mut().unwrap();
				}
				
			} else {
				//Store the node in the output stack, for later access by the now queued child nodes:
				converted_stack.push(ConvertedMeta {
					name: entry.meta.key.take(),
					converted: converted_entry,
					child_count: entry.children.len()
				});
				//Children to process first. Queue them up for processing in the next iteration.
				// Important is to reverse the order. So that the first child gets processed first when being popped from the stack.
				for child in entry.children.into_iter().rev() {
					process_stack.push(child);
				}
			}
		}
		
		let root = converted_stack.pop().unwrap().converted;
		if let JecsType::Map(_) = root {
			Ok(root)
		} else {
			panic!("Impossible to reach code: Something is wrong with the LineContext to JecsType converting code. Did get wrong root type.");
		}
	}
}
