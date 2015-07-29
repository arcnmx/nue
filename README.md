# nue

[![travis-badge][]][travis] [![release-badge][]][cargo] [![docs-badge][]][docs] [![license-badge][]][license]

A collection of tools for working with binary data and POD structs in Rust.

 - [pod][docs-pod] is an approach at building a safe interface for
   transmuting POD structs to and from byte slices.
 - [nue-macros][docs-macros] provides helpers for `pod`, as well
   as a serialization-like library for dealing with binary streams of data.
 - [nue-codegen][docs-codegen] allows the use of `nue-macros` without
   syntax extensions on stable Rust.
 - [nue-io][docs-io] contains various supporting structs and traits for
   readers and writers.


[travis-badge]: https://img.shields.io/travis/arcnmx/nue/master.svg?style=flat-square
[travis]: https://travis-ci.org/arcnmx/nue
[release-badge]: https://img.shields.io/github/release/arcnmx/nue.svg?style=flat-square
[cargo]: https://crates.io/search?q=nue
[docs-badge]: https://img.shields.io/badge/API-docs-blue.svg?style=flat-square
[docs]: http://arcnmx.github.io/nue/nue/
[docs-io]: http://arcnmx.github.io/nue/nue_io/
[docs-pod]: http://arcnmx.github.io/nue/pod/
[docs-macros]: http://arcnmx.github.io/nue/nue_macros/
[docs-codegen]: http://arcnmx.github.io/nue/nue_codegen/
[license-badge]: https://img.shields.io/badge/license-MIT-lightgray.svg?style=flat-square
[license]: https://github.com/arcnmx/nue/blob/master/COPYING
