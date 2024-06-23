# EccJecsLib

A [JECS](https://github.com/JimmyCushnie/JECS) parser library.

Multiple of my Rust projects had to parse JECS files, thus made it a library to use in all of the projects that need it.

You can feed it a file/bytes/text and you get a `JecsType::Map(HashMap<String, JecsType>)`, or an `Error`. Look at the `main.rs` file to see an example usage.

## Changelog:

`v1.0.0`: First version of this repository. Port from the original code written for a LW server project. Error handling was change and general code quality improvements.
`v1.1.0`: Return type of the parser is now a `HashMap<String, JecsType>` instead of a `JecsType`, JECS files "should" always contain Maps as root type (For now that is). Improves usage.
`v1.2.0`: Ignores BOM headers. Fixes boolean interpretation. Simplifies errors, by always returning `std::error::Error`. Parser now returns a map of nodes, instead of a node.