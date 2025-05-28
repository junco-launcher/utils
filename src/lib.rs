/// The `options_parser` module provides functionality for parsing configuration
/// or options data from text input. It includes types and functions to interpret
/// various data types (such as integers, floats, booleans, strings, and lists)
/// from lines of text.
///
/// Typical usage involves calling the parsing functions to convert lines or files
/// into structured data for further processing.
pub mod options_parser;

/// The `mcmeta_parser` module is responsible for parsing `.mcmeta` files,
/// which are commonly used in Minecraft resource packs and data packs to
/// describe metadata such as pack format and description.
///
/// This module provides types and functions to read and interpret `.mcmeta` files
/// into Rust data structures.
pub mod mcmeta_parser;