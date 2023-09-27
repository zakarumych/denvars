# Denvars - Deserialize environment variables

This crate provides deserializer that reads from environment variables or user-provided array of key-value pairs.

For convenience, it can be configured to call specific visiting method for different kind of data.
By default it parses booleans from large set of possible values,\
numbers using `FromStr`,\
sequences from comma-separated values,\
maps from comma-separated key:value pairs,\
allows using potentially escaped strings in double quotes,\
decodes base64-encoded byte arrays if configured (this is default behavior),\
compare uppercased names of fields when deserializing struct from map of env vars if configured (this is default behavior),\
It may treat values as JSON to support deserializing nested structures.\
Custom string parsers may be implemented to support other formats.

## License

Licensed under either of

* Apache License, Version 2.0, ([license/APACHE](license/APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([license/MIT](license/MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
