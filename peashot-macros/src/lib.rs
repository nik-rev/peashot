use itertools::Either;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quip::quip;
use syn::Field;
use syn::Ident;
use syn::ItemStruct;
use syn::parse_macro_input;

#[proc_macro_derive(KdlDocumentCodec)]
pub fn derive_kdl_document_codec(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let structure = parse_macro_input!(input as ItemStruct);

    quip! {
        impl crate::kdl::KdlDocumentCodec for #{structure.ident} {
            fn decode(kdl: kdl::KdlDocument) -> Self {
                for node in kdl {

                }
            }

            fn encode(self) -> kdl::KdlDocument {

            }
        }
    }
    .into()
}

#[proc_macro_derive(KdlNodeCodec, attributes(argument))]
pub fn derive_kdl_node_codec(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut structure = parse_macro_input!(input as ItemStruct);

    let is_children = |field: &&Field| {
        field
            .ident
            .as_ref()
            .expect("only struct with named fields is allowed")
            == "children"
    };

    let children = structure.fields.iter().find(is_children);

    struct M {
        init: TokenStream,
        arm: TokenStream,
    }

    let (arguments, properties): (Vec<_>, Vec<_>) =
        structure.fields.iter().filter(is_children).partition_map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .expect("only struct with named fields is allowed");
            // Whether to treat this field as a positional KDL argument
            let is_argument = field
                .attrs
                .iter()
                .any(|attr| attr.meta.path().is_ident("argument"));
            // Whether to treat this field as KDL Children (whole KDLDocument in curly
            // braces)
            let field_ident = Ident::new(
                &heck::AsKebabCase(field_ident.to_string()).to_string(),
                field_ident.span(),
            );

            if is_argument {
                Either::Left(M {
                    init: quip! { let mut #field_ident = None },
                    arm: quip! {},
                })
            } else {
                Either::Right(M {
                    init: quip! { let mut #field_ident = None },
                    arm: quip! {
                        #{field_ident.to_string()} => {
                            if let Some((prev_span, _)) = #field_ident {
                                errs.emit("duplicate field", entry.span).context("previous definition of field here", prev_span)
                            } else {
                                #field_ident = (entry.span, #{field.ty})
                            }
                        }
                    },
                })
            }
        });
    let arguments_init = arguments.iter().map(|arg| &arg.init);
    let arguments_arms = arguments.iter().map(|arg| &arg.arm);
    let properties_init = properties.iter().map(|prop| &prop.init);
    let properties_arms = properties.iter().map(|prop| &prop.arm);

    quip! {
        impl crate::kdl::KdlNodeCodec for #{structure.ident} {
            fn decode(node: kdl::KdlNode) -> Self {
                if let Some(ty) = node.ty() {
                    error!(ty.span(), "unexpected type annotation");
                }
                #(#properties_init)*
                #(#arguments_init)*

                for entry in node.entries {
                    if let Some(ty) = entry.ty() {
                        error!(ty.span(), "unexpected type annotation");
                    }

                    if let Some(name) = entry.name {
                        match name.value() {
                            #(#properties_arms)*
                            _ => {}
                        }
                    } else {

                    }
                }
                if let Some(children) = node.children {

                }
            }

            fn encode(self) -> kdl::KdlNode {

            }
        }
    }
    .into()
}
