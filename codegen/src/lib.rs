use aster::AstBuilder;
use quasi::ExtParseUtils;
use syntax::ast::{self, MetaItem, MetaItem_, StructField_, Lit_};
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ptr::P;
use syntax::attr;

fn derive_type<'a>(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: &'a Annotatable) ->
    Option<(AstBuilder, &'a P<ast::Item>, ast::Generics, P<ast::Ty>, ast::Path)> {
    let item = match annotatable {
        &Annotatable::Item(ref item) => item,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs or enums");
            return None;
        }
    };

    let generics = match item.node {
        ast::ItemStruct(_, ref generics) => generics,
        ast::ItemEnum(_, ref generics) => generics,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs or enums");
            return None;
        },
    };

    let builder = AstBuilder::new().span(span);

    let generics = builder.from_generics(generics.clone())
    .add_ty_param_bound(
        builder.path().global().ids(&["pod", "Pod"]).build()
    ).build();

    let ty_path = builder.path().segment(item.ident).with_generics(generics.clone()).build().build();
    let ty = builder.ty().build_path(ty_path.clone());

    Some((builder, item, generics, ty, ty_path))
}

fn expand_packed(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: Annotatable) -> Annotatable {
    let item = match annotatable {
        Annotatable::Item(item) => item,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs or enums");
            return annotatable;
        }
    };

    let builder = AstBuilder::new().span(span);

    let repr = builder.attr().list("repr").words(["C"].iter()).build();
    let packed = builder.attr().word("__nue_packed");
    let derive_packed = builder.attr().word("derive_Packed");

    Annotatable::Item(item.map(|mut item| {
        attr::mark_used(&packed);
        item.attrs.push(repr);
        item.attrs.push(packed);
        item.attrs.push(derive_packed);
        item
    }))
}

fn expand_derive_pod_packed(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: Annotatable) -> Annotatable {
    let item = match annotatable {
        Annotatable::Item(item) => item,
        _ => {
            cx.span_err(meta_item.span, "`derive` may only be applied to structs or enums");
            return annotatable;
        }
    };

    let builder = AstBuilder::new().span(span);

    let packed = builder.attr().word("packed");
    let derive_pod = builder.attr().word("derive_Pod");

    Annotatable::Item(item.map(|mut item| {
        item.attrs.push(packed);
        item.attrs.push(derive_pod);
        item
    }))
}

fn expr_is_false(expr: &P<ast::Expr>) -> bool {
    match &expr.node {
        &ast::Expr_::ExprLit(ref lit) if lit.node == Lit_::LitBool(false) => true,
        _ => false,
    }
}

fn expand_derive_packed(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: &Annotatable, push: &mut FnMut(Annotatable)) {
    let (_, item, generics, ty, _) = if let Some(ret) = derive_type(cx, span, meta_item, annotatable) {
        ret
    } else {
        return
    };

    if !item.attrs.iter().any(|a| match &a.node.value.node {
        &MetaItem_::MetaWord(ref name) if *name == "__nue_packed" || *name == "packed" => true,
        _ => false,
    }) {
        cx.span_err(meta_item.span, "packed types require #[packed]");
        return;
    }

    let assertions = match item.node {
        ast::ItemStruct(ref struct_def, _) => {
            struct_def.fields.iter().map(|field| {
                let ty = &field.node.ty;
                quote_stmt!(cx, assert::<$ty>();).unwrap()
            }).collect::<Vec<_>>()
        },
        _ => {
            cx.span_err(meta_item.span, "packed types must be structs");
            return
        },
    };

    let where_clause = &generics.where_clause;

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        unsafe impl $generics ::nue::Packed for $ty $where_clause {
            fn __assert_unaligned() {
                fn assert<T: ::nue::Unaligned>() { }

                $assertions
            }
        }
    ).unwrap();
    push(Annotatable::Item(impl_item));

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        unsafe impl $generics ::nue::Unaligned for $ty $where_clause { }
    ).unwrap();
    push(Annotatable::Item(impl_item));
}

fn expand_derive_pod(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: &Annotatable, push: &mut FnMut(Annotatable)) {
    let (_, item, generics, ty, _) = if let Some(ret) = derive_type(cx, span, meta_item, annotatable) {
        ret
    } else {
        return
    };

    if !item.attrs.iter().any(|a| match &a.node.value.node {
        &MetaItem_::MetaWord(ref name) if *name == "__nue_packed" || *name == "packed" => true,
        _ => false,
    }) {
        cx.span_err(meta_item.span, "POD types require #[packed]");
        return;
    }

    let assertions = match item.node {
        ast::ItemStruct(ref struct_def, _) => {
            struct_def.fields.iter().map(|field| {
                let ty = &field.node.ty;
                quote_stmt!(cx, assert::<$ty>();).unwrap()
            }).collect::<Vec<_>>()
        },
        _ => {
            cx.span_err(meta_item.span, "POD types must be structs");
            return
        },
    };

    let where_clause = &generics.where_clause;

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        unsafe impl $generics ::pod::Pod for $ty $where_clause {
            fn __assert_pod() {
                fn assert<T: ::pod::Pod>() { }

                $assertions
            }
        }
    ).unwrap();

    push(Annotatable::Item(impl_item))
}

fn expand_derive_encode(cx: &mut ExtCtxt, span: Span, meta_item: &MetaItem, annotatable: &Annotatable, push: &mut FnMut(Annotatable)) {
    let (builder, item, generics, ty, _) = if let Some(ret) = derive_type(cx, span, meta_item, annotatable) {
        ret
    } else {
        return
    };

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

                let statement = quote_stmt!(cx,
                    let _ = try!(::nue::Encode::encode($expr, __w));
                ).unwrap();
                let mut statement = vec![statement];

                for attr in field_attrs(cx, field, "nue_enc", false) {
                    match attr {
                        FieldAttribute::Cond(expr) => cond = Some(expr),
                        FieldAttribute::Default(_) => (),
                        FieldAttribute::Align(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx, let _ = try!(::nue::SeekAlignExt::align_to(__w, $expr)); ).unwrap());
                        },
                        FieldAttribute::Skip(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx,
                                let _ = try!(::nue::SeekForward::seek_forward(__w, $expr));
                            ).unwrap());
                        },
                        FieldAttribute::Limit(expr) => statement.insert(0, quote_stmt!(cx, let __w = &mut ::nue::Take::new(::std::borrow::BorrowMut::borrow_mut(__w), $expr); ).unwrap()),
                        FieldAttribute::Consume(expr) => statement.push(quote_stmt!(cx,
                            if $expr {
                                let _ = try!(match ::std::io::copy(&mut ::std::io::repeat(0), __w) {
                                    ::std::result::Result::Err(ref err) if err.kind() == ::std::io::ErrorKind::WriteZero => Ok(0),
                                    res => res,
                                });
                            }
                        ).unwrap()),
                        FieldAttribute::Assert(expr) => statement.insert(0, quote_stmt!(cx,
                            if !$expr {
                                return Err(::std::io::Error::new(::std::io::ErrorKind::InvalidInput, concat!("assertion ", stringify!($expr), " failed")));
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
        ast::ItemEnum(..) => unimplemented!(),
        _ => {
            cx.span_err(meta_item.span, "`derive` must be used on structs and enums");
            return;
        },
    };

    let needs_seek = if needs_seek {
        quote_stmt!(cx,
            let __w = &mut ::nue::ReadWriteTell::new(::nue::SeekForwardWrite::new(::nue::SeekAll::new(__w)));
        )
    } else {
        quote_stmt!(cx, let __w = &mut ::nue::SeekAll::new(__w);)
    }.unwrap();

    let where_clause = &generics.where_clause;

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        impl $generics ::nue::Encode for $ty $where_clause {
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
    let (builder, item, generics, ty, ty_path) = if let Some(ret) = derive_type(cx, span, meta_item, annotatable) {
        ret
    } else {
        return
    };

    let mut needs_seek = false;
    let mut tuple_struct = false;

    let (decoders, decoder_fields) = match item.node {
        ast::ItemStruct(ref struct_def, _) => {
            struct_def.fields.iter().enumerate().map(|(i, field)| {
                let field = &field.node;
                let (let_name, field_name) = match field.kind {
                    ast::NamedField(name, _) => (builder.id(format!("__self_0{}", name)), Some(name)),
                    ast::UnnamedField(_) => {
                        tuple_struct = true;
                        (builder.id(format!("__self_0{}", i)), None)
                    },
                };

                let (mut cond, mut cond_default) = (None, None);
                let field_type = &field.ty;

                let statement = quote_stmt!(cx,
                    let $let_name: $field_type = try!(::nue::Decode::decode(__r));
                ).unwrap();
                let mut statement = vec![statement];

                for attr in field_attrs(cx, field, "nue_dec", true) {
                    match attr {
                        FieldAttribute::Cond(expr) => cond = Some(expr),
                        FieldAttribute::Default(expr) => cond_default = Some(expr),
                        FieldAttribute::Align(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx, let _ = try!(::nue::SeekAlignExt::align_to(__r, $expr)); ).unwrap());
                        },
                        FieldAttribute::Skip(expr) => {
                            needs_seek = true;
                            statement.insert(0, quote_stmt!(cx,
                                let _ = try!(::nue::SeekForward::seek_forward(__r, $expr));
                            ).unwrap());
                        },
                        FieldAttribute::Limit(expr) => statement.insert(0, quote_stmt!(cx, let __r = &mut ::nue::Take::new(::std::borrow::BorrowMut::borrow_mut(__r), $expr); ).unwrap()),
                        FieldAttribute::Consume(expr) => statement.push(quote_stmt!(cx,
                            if $expr {
                                let _ = try!(::std::io::copy(__r, &mut ::std::io::sink()));
                            }
                        ).unwrap()),
                        FieldAttribute::Assert(expr) => statement.push(quote_stmt!(cx,
                            if !$expr {
                                return Err(::std::io::Error::new(::std::io::ErrorKind::InvalidInput, concat!("assertion ", stringify!($expr), " failed")));
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
        ast::ItemEnum(..) => {
            cx.span_err(meta_item.span, "enums cannot be decoded");
            return;
        },
        _ => {
            cx.span_err(meta_item.span, "`derive` must be used on structs and enums");
            return;
        },
    };

    let needs_seek = if needs_seek {
        quote_stmt!(cx,
            let __r = &mut ::nue::ReadWriteTell::new(::nue::SeekForwardRead::new(::nue::SeekAll::new(__r)));
        )
    } else {
        quote_stmt!(cx, let __r = &mut ::nue::SeekAll::new(__r);)
    }.unwrap();

    let result = if tuple_struct {
        builder.expr().call().build_path(ty_path).with_args(decoder_fields.into_iter().map(|(let_name, _)| builder.expr().id(let_name))).build()
    } else {
        builder.expr().struct_path(ty_path).with_id_exprs(decoder_fields.into_iter().map(|(let_name, field_name)| (field_name.unwrap(), builder.expr().id(let_name)))).build()
    };

    let where_clause = &generics.where_clause;

    let impl_item = quote_item!(cx,
        #[automatically_derived]
        impl $generics ::nue::Decode for $ty $where_clause {
            type Options = ();

            fn decode<__R: ::std::io::Read>(__r: &mut __R) -> ::std::io::Result<Self> {
                $needs_seek
                $decoders
                let __result = $result;

                let _ = try!(::nue::Decode::validate(&__result));

                Ok(__result)
            }
        }
    ).unwrap();

    push(Annotatable::Item(impl_item));
}

fn field_attrs(cx: &mut ExtCtxt, field: &StructField_, meta_name: &'static str, replace_self: bool) -> Vec<FieldAttribute> {
    fn attr_expr(cx: &mut ExtCtxt, replace_self: bool, value: &str) -> P<ast::Expr> {
        let value = if replace_self {
            value.replace("self.", "__self_0")
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
                    "assert" => attrs.push(FieldAttribute::Assert(attr_expr(cx, replace_self, &value))),
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
                    cx.span_err(attr.span, "invalid attribute");
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
    Assert(P<ast::Expr>),
}
