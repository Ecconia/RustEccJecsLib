use ecc_ansi_lib::ansi;
use crate::types::JecsType;

pub fn debug_print(entry: &JecsType) {
	print_inner(entry,
		ansi!("«gr»└ ").to_owned(),
		ansi!("«gr»  ").to_owned(),
	);
}

fn print_inner(entry: &JecsType, entry_prefix: String, prefix: String) {
	match entry {
		JecsType::Any() => {
			println!(ansi!("«y»{}«r»{}«»"), entry_prefix, "---");
		}
		JecsType::Value(value) => {
			println!(ansi!("{}'«w»{}«gr»'«»"), entry_prefix, value);
		}
		JecsType::Map(map) => {
			println!(ansi!("{}<map>«»"), entry_prefix);
			for (index, (key, value)) in map.iter().enumerate() {
				print_inner(value,
					format!(ansi!("{}{} «w»{}«gr»: "),
						prefix, if index == (map.len() - 1) { '└' } else { '├' }, key
					),
					format!("{}{} ",
						prefix, if index == (map.len() - 1) { ' ' } else { '│' }
					),
				);
			}
		}
		JecsType::List(list) => {
			println!(ansi!("«y»{}<list>«»"), entry_prefix);
			for (index, value) in list.iter().enumerate() {
				print_inner(value,
					format!("{}{} ",
						prefix, if index == (list.len() - 1) { '└' } else { '├' }
					),
					format!("{}{} ",
						prefix, if index == (list.len() - 1) { ' ' } else { '│' }
					),
				);
			}
		}
	}
}