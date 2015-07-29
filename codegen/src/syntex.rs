#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private, plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(quasi_macros))]
#![deny(missing_docs)]

//! nue derive syntax extension.
//!
//! Provides the `#[derive(PodPacked, Pod, NueEncode, NueDecode)]` extensions documented in `nue-macros`.
//!
//! ## Stable
//!
//! See the [syntex documentation](https://github.com/erickt/rust-syntex/blob/master/README.md)
//! for instructions on how to set up your project to use these macros in stable Rust.
//!
//! ## Nightly / Unstable
//!
//! See the example in `nue-macros` for usage as a normal syntax extension.

extern crate aster;
extern crate quasi;
#[cfg(feature = "with-syntex")]
extern crate syntex;
#[cfg(feature = "with-syntex")]
extern crate syntex_syntax as syntax;
#[cfg(not(feature = "with-syntex"))]
extern crate syntax;
#[cfg(not(feature = "with-syntex"))]
extern crate rustc;

#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/lib.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("lib.rs");

/// Registers the plugin for expansion with syntex.
#[cfg(feature = "with-syntex")]
pub fn register(reg: &mut syntex::Registry) {
    use syntax::{ast, fold};

    reg.add_attr("feature(custom_derive)");
    reg.add_attr("feature(custom_attribute)");

    reg.add_modifier("packed", expand_packed);
    reg.add_modifier("derive_PodPacked", expand_derive_pod_packed);
    reg.add_decorator("derive_Packed", expand_derive_packed);
    reg.add_decorator("derive_Pod", expand_derive_pod);
    reg.add_decorator("derive_NueEncode", expand_derive_encode);
    reg.add_decorator("derive_NueDecode", expand_derive_decode);

    reg.add_post_expansion_pass(strip_attributes);

    #[cfg(feature = "with-syntex")]
    fn strip_attributes(krate: ast::Crate) -> ast::Crate {
        struct StripAttributeFolder;

        impl fold::Folder for StripAttributeFolder {
            fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
                match attr.node.value.node {
                    ast::MetaWord(ref n) if *n == "__nue_packed" => { return None; },
                    ast::MetaList(ref n, _) if *n == "nue" || *n == "nue_enc" || *n == "nue_dec" => { return None; },
                    _ => {}
                }

                Some(attr)
            }

            fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
                fold::noop_fold_mac(mac, self)
            }
        }

        fold::Folder::fold_crate(&mut StripAttributeFolder, krate)
    }
}

#[doc(hidden)]
#[cfg(not(feature = "with-syntex"))]
pub fn register(reg: &mut rustc::plugin::Registry) {
    reg.register_syntax_extension(
        syntax::parse::token::intern("packed"),
        syntax::ext::base::MultiModifier(
            Box::new(expand_packed)
        )
    );

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Packed"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_packed)
        )
    );

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_PodPacked"),
        syntax::ext::base::MultiModifier(
            Box::new(expand_derive_pod_packed)
        )
    );

    reg.register_syntax_extension(
        syntax::parse::token::intern("derive_Pod"),
        syntax::ext::base::MultiDecorator(
            Box::new(expand_derive_pod)
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
