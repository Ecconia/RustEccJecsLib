use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::str::Utf8Error;

use ecc_jecs_lib::{debug, parser};
use ecc_jecs_lib::errors::JecsCorruptedDataError;

fn main() {
	//This file serves as a test for myself to check if my parser does not break on all files of my installation.
	let test_path = Path::new("/home/ecconia/.steam/steam/steamapps/common/Logic World/");
	let mut files = Vec::new();
	walk_folder(test_path, &mut |path| {
		let file_name = path.file_name().unwrap().to_str().unwrap();
		//SECCs is the old name for JECS. Will be obsolete once Logic World updates the naming to JECS.
		if file_name.ends_with(".succ") || file_name.ends_with(".jecs") {
			files.push(path);
		}
	});
	
	for file in files {
		println!("- {}", file.to_str().unwrap());
		let tree = match parser::parse_jecs_file(&file) {
			Ok(tree) => tree,
			Err(e) => {
				if let Some(e) = e.downcast_ref::<std::io::Error>() {
					panic!("Could not read manifest file! Error: {}", e);
				} else if let Some(e) = e.downcast_ref::<Utf8Error>() {
					panic!("Manifest file does not contain valid UTF-8! Error: {}", e);
				} else if let Some(e) = e.downcast_ref::<JecsCorruptedDataError>() {
					panic!("Manifest has invalid content: {}", e);
				} else {
					panic!("Unknown exception: {}", e);
				}
			}
		};
		debug::debug_print(&tree);
	}
}

fn walk_folder(path: &Path, function: &mut dyn FnMut(PathBuf)) {
	for entry in read_dir(path).unwrap() {
		let entry = entry.unwrap();
		let entry_path = entry.path();
		if entry_path.is_dir() {
			let sym_meta = entry_path.symlink_metadata().unwrap();
			if sym_meta.is_symlink() {
				continue;
			}
			walk_folder(&entry_path, function);
		} else if entry_path.is_file() {
			function(entry_path.to_owned());
		}
	}
}
