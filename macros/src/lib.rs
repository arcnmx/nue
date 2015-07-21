#![feature(plugin_registrar, rustc_private, plugin)]
#![plugin(quasi_macros)]

//! `#[derive(..)]` attributes for POD and binary encodable types.
//!
//! # Attributes
//!
//! ## `#[derive(Pod)]`
//!
//! Marks a struct as `pod::Pod`. It must only contain other `Pod` members, and
//! the derive will fail unless an explicit `repr` is applied to the type.
//!
//! ### `#[derive(PodReprPacked)]`
//!
//! Marks a struct as both `#[repr(packed)]` and `pod::Pod`.
//!
//! ### `#[derive(PodReprC)]`
//!
//! Similar to `PodReprPacked` but uses `#[repr(C)]` instead. Note that this
//! allows access to member padding, which may be unsafe uninitialized data.
//!
//! ### Example
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate pod;
//! use pod::{Encode, PodExt};
//!
//! # fn main() {
//! #[derive(PodReprPacked)]
//! struct Data(u8);
//!
//! assert_eq!(Data(5).as_slice(), &[5]);
//! # }
//! ```
//!
//! ## `#[derive(NueEncode, NueDecode)]`
//!
//! Implements `pod::Encode` and `pod::Decode` on the struct.
//! All fields must also implement `Encode` / `Decode` (or be skipped by a `nue` attribute).
//!
//! ### `#[nue(...)]`, `#[nue_enc(...)]`, `#[nue_dec(...)]`
//!
//! Additional coding options may be provided per field using the `nue` attributes.
//! They will affect how the parent type is encoded or decoded. The attributes accept
//! arbitrary Rust expressions. Member variables may be accessed through `self`.
//! 
//! `nue_enc` only applies to encoding, `nue_dec` applies to decoding, and `nue` applies to both.
//! The order of the attributes doesn't usually matter, though `align` and `skip` interact
//! differently depending on which is defined first.
//!
//! #### `align`
//!
//! Aligns the field to an offset of the given multiple.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate pod; extern crate nue_io;
//! use pod::Encode;
//!
//! # fn main() {
//! #[derive(NueEncode)]
//! struct Data(
//! 	u8,
//! 	#[nue(align = "self.0 as u64 + 1")]
//! 	&'static str
//! );
//!
//! let data = Data(2, "hi");
//! let cmp = &[2, 0, 0, b'h', b'i'];
//! assert_eq!(&data.encode_vec().unwrap(), cmp);
//! # }
//! ```
//!
//! #### `skip`
//!
//! Discards the provided amount of bytes before encoding/decoding the value.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate pod; extern crate nue_io;
//! use pod::Encode;
//!
//! # fn main() {
//! #[derive(NueEncode)]
//! struct Data(
//! 	u8,
//! 	#[nue(skip = "1")]
//! 	&'static str
//! );
//!
//! let data = Data(2, "hi");
//! let cmp = &[2, 0, b'h', b'i'];
//! assert_eq!(&data.encode_vec().unwrap(), cmp);
//! # }
//! ```
//!
//! #### `cond`
//!
//! Conditionally encodes or decodes the field. If the condition is not met,
//! `Default::default()` will be used.
//! `false` is a static guarantee that the field will be ignored.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate pod; extern crate nue_io;
//! use pod::Encode;
//!
//! # fn main() {
//! #[derive(NueEncode)]
//! struct Data<'a>(
//! 	u8,
//! 	#[nue(cond = "false")]
//! 	&'a () // Note that this type does not implement `Encode`
//! );
//!
//! let u = ();
//! let data = Data(2, &u);
//! let cmp = &[2];
//! assert_eq!(&data.encode_vec().unwrap(), cmp);
//! # }
//! ```
//!
//! #### `default`
//!
//! Determines the default value to be used if `cond` evaluates to false.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate pod; extern crate nue_io;
//! use pod::Decode;
//!
//! # fn main() {
//! #[derive(NueDecode, PartialEq, Debug)]
//! struct Data(
//! 	u8,
//! 	#[nue(cond = "self.0 == 1", default = "5")]
//! 	u8
//! );
//!
//! let data = &[2];
//! assert_eq!(&Data::decode_slice(data).unwrap(), &Data(2, 5));
//!
//! let data = &[1, 2];
//! assert_eq!(&Data::decode_slice(data).unwrap(), &Data(1, 2));
//! # }
//! ```
//!
//! #### `limit`
//!
//! Limits the amount of bytes that can be consumed or written during coding.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate pod; extern crate nue_io;
//! use pod::Decode;
//!
//! # fn main() {
//! #[derive(NueDecode)]
//! struct Data(
//! 	#[nue(limit = "4")]
//! 	String,
//! );
//!
//! let data = b"hello";
//! assert_eq!(&Data::decode_slice(data).unwrap().0, "hell");
//! # }
//! ```
//!
//! #### `consume`
//!
//! When set, uses all of `limit` even if the type did not encode or decode the entire byte region.
//!
//! ```
//! # #![feature(plugin, custom_derive, custom_attribute)] #![plugin(nue_macros)]
//! # extern crate pod; extern crate nue_io;
//! use pod::Decode;
//! use std::ffi::CString;
//!
//! # fn main() {
//! #[derive(NueDecode)]
//! struct Data(
//! 	#[nue(limit = "8", consume = "true")]
//! 	CString,
//! 	u8
//! );
//!
//! let data = b"hello\0\0\0\x05";
//! let data = Data::decode_slice(data).unwrap();
//! assert_eq!(data.0.to_bytes(), b"hello");
//! assert_eq!(data.1, 5);
//! # }
//! ```

extern crate rustc;
extern crate quasi;
extern crate aster;
extern crate syntax;

use aster::AstBuilder;
use quasi::ExtParseUtils;
use syntax::ast::{self, MetaItem, MetaItem_, StructField_, Lit_};
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::attr;

#[plugin_registrar]
#[doc(hidden)]
pub fn register(reg: &mut rustc::plugin::Registry) {
    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_PodReprPacked"),
        syntax::ext::base::MultiModifier(
            Box::new(expand_derive_pod_repr_packed)
        )
    );

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_PodReprC"),
        syntax::ext::base::MultiModifier(
            Box::new(expand_derive_pod_repr_c)
        )
    );

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Pod"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_pod_repr)
        )
    );

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_NueEncode"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_encode)
        )
    );

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_NueDecode"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_decode)
        )
    );
}

fn expand_derive_pod_repr_packed(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: Annotatable) -> Annotatable {
    expand_derive_pod_repr_kind(cx, span, meta_item, annotatable, "packed")
}

fn expand_derive_pod_repr_c(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: Annotatable) -> Annotatable {
    expand_derive_pod_repr_kind(cx, span, meta_item, annotatable, "C")
}

fn expand_derive_pod_repr_kind(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: Annotatable, kind: &'static str) -> Annotatable {
    let item = match annotatable {
        Annotatable::Item(item) => item,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs");
            return annotatable;
        }
    };

    let builder = AstBuilder::new().span(span);

    let repr = builder.attr().list("repr").words([kind].iter()).build();
    let derive_pod_repr = builder.attr().word("derive_Pod");

    Annotatable::Item(item.map(|mut item| {
        item.attrs.push(repr);
        item.attrs.push(derive_pod_repr);
        item
    }))
}

fn expr_is_false(expr: &P<ast::Expr>) -> bool {
    match &expr.node {
        &ast::Expr_::ExprLit(ref lit) if lit.node == Lit_::LitBool(false) => true,
        _ => false,
    }
}

fn expand_derive_pod_repr(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: &Annotatable, push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs");
            return;
        }
    };

    let generics = match item.node {
        ast::ItemStruct(_, ref generics) => generics,
        ast::ItemEnum(_, _) => {
            cx.span_err(meta_item.span, "only structs may be POD");
            return;
        },
        _ => cx.bug("expected ItemStruct or ItemEnum")
    };

    if !item.attrs.iter().any(|a| match &a.node.value.node {
        &MetaItem_::MetaList(ref name, _) if *name == "repr" => true,
        _ => false,
    }) {
        cx.span_err(meta_item.span, "POD types require #[repr(..)]");
        return;
    }

    let builder = AstBuilder::new().span(span);

    let impl_generics = builder.from_generics(generics.clone())
    .add_ty_param_bound(
        builder.path().global().ids(&["pod", "PodType"]).build()
    ).build();

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let where_clause = &impl_generics.where_clause;

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        impl $impl_generics ::pod::Pod for $ty $where_clause { }
    ).unwrap();

    push(Annotatable::Item(impl_item))
}

fn expand_derive_encode(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: &Annotatable, push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs");
            return;
        }
    };

    let generics = match item.node {
        ast::ItemStruct(_, ref generics) => generics,
        ast::ItemEnum(_, _) => {
            cx.span_err(meta_item.span, "only structs may be POD");
            return;
        },
        _ => cx.bug("expected ItemStruct or ItemEnum")
    };

    let builder = AstBuilder::new().span(span);

    let impl_generics = builder.from_generics(generics.clone())
    .add_ty_param_bound(
        builder.path().global().ids(&["pod", "Encode"]).build()
    ).build();

    let ty = builder.ty().path()
        .segment(item.ident).with_generics(impl_generics.clone()).build()
        .build();

    let mut needs_seek = false;

    let encoders = match item.node {
        ast::ItemStruct(ref struct_def, _) => {
            struct_def.fields.iter().enumerate().map(|(i, field)| {
                let field = &field.node;
                let expr = match field.kind {
                    ast::NamedField(name, _) => quote_expr!(cx, &self.$name),
                    ast::UnnamedField(_) => builder.expr().addr_of().tup_field(i).build(builder.expr().self_()),
                };

                let mut cond = None;

                let mut statement = vec![
                    quote_stmt!(cx,
                        let _ = try!(::pod::Encode::encode($expr, __w));
                    ).unwrap()
                ];

                for attr in field_attrs(cx, field, "nue_enc", false) {
                    match attr {
                        FieldAttribute::Cond(expr) => cond = Some(expr),
                        FieldAttribute::Default(_) => (),
                        FieldAttribute::Align(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx, let _ = try!(::nue_io::SeekAlignExt::align_to(__w, $expr)); ).unwrap());
                        },
                        FieldAttribute::Skip(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx,
                                let _ = try!(::std::io::Seek::seek(__w, ::std::io::SeekFrom::Current($expr)));
                            ).unwrap());
                        },
                        FieldAttribute::Limit(expr) => statement.insert(0, quote_stmt!(cx, let __w = &mut ::nue_io::Take::new(::std::io::Write::by_ref(__w), $expr); ).unwrap()),
                        FieldAttribute::Consume(expr) => statement.push(quote_stmt!(cx,
                            if $expr {
                                let _ = try!(match ::std::io::copy(&mut ::std::io::repeat(0), __w) {
                                    ::std::result::Result::Err(ref err) if err.kind() == ::std::io::ErrorKind::WriteZero => Ok(0),
                                    res => res,
                                });
                            }
                        ).unwrap()),
                    }
                }

                if let Some(cond) = cond {
                    if expr_is_false(&cond) {
                        quote_stmt!(cx, {}).unwrap()
                    } else {
                        quote_stmt!(cx,
                            if $cond {
                                $statement
                            }
                        ).unwrap()
                    }
                } else {
                    quote_stmt!(cx, { $statement }).unwrap()
                }
            }).collect::<Vec<_>>()
        },
        _ => panic!()
    };

    let where_clause = &impl_generics.where_clause;

    let needs_seek = if needs_seek {
        quote_stmt!(cx,
            let __w = &mut ::nue_io::SeekForward::seek_forward(__w);
        )
    } else {
        quote_stmt!(cx, {})
    }.unwrap();

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        impl $impl_generics ::pod::Encode for $ty $where_clause {
            type Options = ();

            fn encode<__W: ::std::io::Write>(&self, __w: &mut __W) -> ::std::io::Result<()> {
                $needs_seek
                $encoders

                Ok(())
            }
        }
    ).unwrap();

    push(Annotatable::Item(impl_item));
}

fn expand_derive_decode(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: &Annotatable, push: &mut FnMut(Annotatable)) {
    let item = match *annotatable {
        Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs");
            return;
        }
    };

    let generics = match item.node {
        ast::ItemStruct(_, ref generics) => generics,
        ast::ItemEnum(_, ref generics) => generics,
        _ => cx.bug("expected ItemStruct or ItemEnum")
    };

    let builder = AstBuilder::new().span(span);

    let impl_generics = builder.from_generics(generics.clone())
    .add_ty_param_bound(
        builder.path().global().ids(&["pod", "Decode"]).build()
    ).build();

    let ty_path = builder.path().segment(item.ident).with_generics(impl_generics.clone()).build().build();
    let ty = builder.ty().build_path(ty_path.clone());

    let mut needs_seek = false;
    let mut tuple_struct = false;

    let (decoders, decoder_fields) = match item.node {
        ast::ItemStruct(ref struct_def, _) => {
            struct_def.fields.iter().enumerate().map(|(i, field)| {
                let field = &field.node;
                let (let_name, field_name) = match field.kind {
                    ast::NamedField(name, _) => (builder.id(format!("__field{}", name)), Some(name)),
                    ast::UnnamedField(_) => {
                        tuple_struct = true;
                        (builder.id(format!("__field{}", i)), None)
                    },
                };

                let (mut cond, mut cond_default) = (None, None);

                let mut statement = vec![
                    quote_stmt!(cx,
                        let $let_name = try!(::pod::Decode::decode(__r));
                    ).unwrap()
                ];

                for attr in field_attrs(cx, field, "nue_dec", true) {
                    match attr {
                        FieldAttribute::Cond(expr) => cond = Some(expr),
                        FieldAttribute::Default(expr) => cond_default = Some(expr),
                        FieldAttribute::Align(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx, let _ = try!(::nue_io::SeekAlignExt::align_to(__r, $expr)); ).unwrap());
                        },
                        FieldAttribute::Skip(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx,
                                let _ = try!(::std::io::Seek::seek(__r, ::std::io::SeekFrom::Current($expr)));
                            ).unwrap());
                        },
                        FieldAttribute::Limit(expr) => statement.insert(0, quote_stmt!(cx, let __r = &mut ::nue_io::Take::new(::std::io::Read::by_ref(__r), $expr); ).unwrap()),
                        FieldAttribute::Consume(expr) => statement.push(quote_stmt!(cx,
                            if $expr {
                                let _ = try!(::std::io::copy(__r, &mut ::std::io::sink()));
                            }
                        ).unwrap()),
                    }
                }

                let statement = if let Some(cond) = cond {
                    let default = cond_default.unwrap_or_else(|| quote_expr!(cx, ::std::default::Default::default()));

                    if expr_is_false(&cond) {
                        quote_stmt!(cx, let $let_name = $default;).unwrap()
                    } else {
                        quote_stmt!(cx,
                            let $let_name = if $cond {
                                $statement;
                                $let_name
                            } else {
                                $default
                            };
                        ).unwrap()
                    }
                } else {
                    quote_stmt!(cx, let $let_name = { $statement; $let_name };).unwrap()
                };

                (statement, (let_name, field_name))
            }).unzip::<_, _, Vec<_>, Vec<_>>()
        },
        _ => panic!()
    };

    let where_clause = &impl_generics.where_clause;

    let needs_seek = if needs_seek {
        quote_stmt!(cx,
            let __r = &mut ::nue_io::SeekForward::seek_forward(__r);
        )
    } else {
        quote_stmt!(cx, {})
    }.unwrap();

    let result = if tuple_struct {
        builder.expr().call().build_path(ty_path).with_args(decoder_fields.into_iter().map(|(let_name, _)| builder.expr().id(let_name))).build()
    } else {
        builder.expr().struct_path(ty_path).with_id_exprs(decoder_fields.into_iter().map(|(let_name, field_name)| (field_name.unwrap(), builder.expr().id(let_name)))).build()
    };

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        impl $impl_generics ::pod::Decode for $ty $where_clause {
            type Options = ();

            fn decode<__R: ::std::io::Read>(__r: &mut __R) -> ::std::io::Result<Self> {
                $needs_seek
                $decoders

                Ok($result)
            }
        }
    ).unwrap();

    push(Annotatable::Item(impl_item));
}

fn field_attrs(cx: &mut ExtCtxt, field: &StructField_, meta_name: &'static str, replace_self: bool) -> Vec<FieldAttribute> {
    fn attr_expr(cx: &mut ExtCtxt, replace_self: bool, value: &str) -> P<ast::Expr> {
        let value = if replace_self {
            value.replace("self.", "__field")
        } else {
            value.into()
        };
        cx.parse_expr(value)
    }

    let attr = field.attrs.iter().filter_map(|v| match &v.node.value.node {
        &MetaItem_::MetaList(ref name, ref attrs) if *name == meta_name || *name == "nue" => {
            attr::mark_used(v);

            Some(attrs)
        },
        _ => None,
    });


    let mut attrs = Vec::new();
    for attr in attr {
        for attr in attr.iter() {
            match &attr.node {
                &MetaItem_::MetaNameValue(ref name, Spanned { node: Lit_::LitStr(ref value, _), .. } ) => match &**name {
                    "align" => attrs.push(FieldAttribute::Align(attr_expr(cx, replace_self, &value))),
                    "skip" => attrs.push(FieldAttribute::Skip(attr_expr(cx, replace_self, &value))),
                    "limit" => attrs.push(FieldAttribute::Limit(attr_expr(cx, replace_self, &value))),
                    "cond" => attrs.push(FieldAttribute::Cond(attr_expr(cx, replace_self, &value))),
                    "default" => attrs.push(FieldAttribute::Default(attr_expr(cx, replace_self, &value))),
                    "consume" => attrs.push(FieldAttribute::Consume(attr_expr(cx, replace_self, &value))),
                    _ => {
                        cx.span_err(attr.span, "invalid attribute key");
                        break
                    },
                },
                _ => {
                    cx.span_err(attr.span, "format error");
                    break
                },
            }
        }
    }
    attrs
}

enum FieldAttribute {
    Cond(P<ast::Expr>),
    Default(P<ast::Expr>),
    Align(P<ast::Expr>),
    Limit(P<ast::Expr>),
    Skip(P<ast::Expr>),
    Consume(P<ast::Expr>),
}

// #[derive(PodRepr)]
// Adds repr(packed) and derives PodRepr. Error if the struct does not impl Pod.

// #[derive(Decodable)]
// impl Decodable per field
// impl !Pod

// #[derive(Encodable)]
// impl Encodable per field
// impl !Pod

// #[binenc()]
// Field and struct attributes:
// - endian: "big", "little", "native" (default)
//
// Field attributes:
// - align: "expr"
// - len: "expr"
//    applies to: Vec, String
//    default: None (error for above types?)
// - max: "expr"
// - if: "expr"
//    applies to: anything with Default
// - skip: "expr"

// Procedural/library alternative helpers for all of the above features
