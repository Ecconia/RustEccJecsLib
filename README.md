# EccJecsLib

A [JECS](https://github.com/JimmyCushnie/JECS) parser library.

Multiple of my Rust projects had to parse JECS files, thus made it a library to use in all of the projects that need it.

You can feed it a file/bytes/text and you get a `JecsType::Map(HashMap<String, JecsType>)`, or an `Error`. Look at the `main.rs` file to see an example usage.
