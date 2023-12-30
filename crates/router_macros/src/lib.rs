use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, FnArg, ItemFn, LitStr,
    Pat, PatType, Path, ReturnType, Token, Type, Visibility,
};

enum Segment {
    Literal(String),
    Variable(Ident),
}

impl Parse for Segment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            let segment: LitStr = input.parse()?;
            return Ok(Self::Literal(segment.value()));
        }

        if input.peek(syn::Ident) {
            let segment = input.parse()?;
            return Ok(Self::Variable(segment));
        }

        Err(input.error("Cannot parse route segment"))
    }
}

struct Segments {
    segments: Vec<Segment>,
    glob: Option<Ident>,
}

impl Parse for Segments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut segments = Vec::new();

        loop {
            if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                let segment = input.parse()?;
                return Ok(Self {
                    segments,
                    glob: Some(segment),
                });
            }

            if input.is_empty() {
                return Ok(Self {
                    segments,
                    glob: None,
                });
            }

            segments.push(input.parse()?);

            if !input.is_empty() {
                input.parse::<Token![/]>()?;
            }
        }
    }
}

impl Segments {
    fn conditions(&self) -> TokenStream {
        self.segments
            .iter()
            .enumerate()
            .map(|(index, segment)| match segment {
                Segment::Literal(segment) => quote! {
                    if !matches!(req.segments.get(#index), Some(&#segment)) {
                        return None;
                    }
                },
                Segment::Variable(ident) => quote! {
                    let Some(#ident) = req.segments.get(#index) else {
                        return None;
                    };
                },
            })
            .chain(std::iter::once({
                let len = self.segments.len();
                self.glob
                    .as_ref()
                    .map(|ident| {
                        quote! {
                            let #ident = &req.segments[#len..];
                        }
                    })
                    .unwrap_or_else(|| {
                        quote! {
                            if req.segments.len() != #len {
                                return None;
                            }
                        }
                    })
            }))
            .collect()
    }
}

fn route_attribute(
    segments: Segments,
    handler: ItemFn,
    verb: Option<TokenStream>,
) -> proc_macro2::TokenStream {
    let conditions = segments.conditions();
    let method_condition = verb.map(|verb| {
        quote! {
            if req.request.method() != #verb {
                return None;
            }
        }
    });

    let conditions = quote! {
        #method_condition
        #conditions
    };

    let arguments = handler.sig.inputs.iter().skip(1).map(|arg| {
        if let FnArg::Typed(PatType { pat, .. }) = arg {
            if let Pat::Ident(ident) = pat.as_ref() {
                return quote!(#ident);
            }
        }
        panic!("Unexpected argument")
    });

    if handler.sig.generics.lt_token.is_some() {
        panic!("Handlers cannot contain generics");
    };

    let request = handler
        .sig
        .inputs
        .first()
        .and_then(|arg| match arg {
            FnArg::Typed(PatType { ty, .. }) => match ty.as_ref() {
                Type::Reference(ty) => Some(&ty.elem),
                _ => None,
            },
            _ => None,
        })
        .expect("Must have a reference to a request as the first parameter");

    let ReturnType::Type(_, response) = handler.sig.output.clone() else {
        panic!("Must have reponse as return type");
    };

    let handler_mod_ident = handler.sig.ident.clone();
    let visibility = handler.vis.clone();

    let asyncness = handler.sig.asyncness;

    let mut handler = handler.clone();
    handler.vis = Visibility::Public(syn::token::Pub::default());
    handler.sig.generics = parse_quote!(<'req>);
    handler.sig.ident = format_ident!("__{handler_mod_ident}");
    let handler_ident = &handler.sig.ident;

    let handler_request = format_ident!("Request__{}", handler_mod_ident);
    let handler_response = format_ident!("Response__{}", handler_mod_ident);

    let matched_segments = segments.segments.len();

    let try_match = if asyncness.is_some() {
        quote! {
            pub async fn try_match_async<'req>(
                req: &'req ::router::Request<'req, RequestBody<'req>, RequestContext<'req>>
            ) -> Option<::router::http::Response<ResponseBody<'req>>> {
                #conditions

                let req = req.ignore_segments(#matched_segments);
                let result = handler(&req, #(#arguments),*).await;
                Some(result)
            }
        }
    } else {
        quote! {
            pub fn try_match<'req>(
                req: &'req ::router::Request<'req, RequestBody<'req>, RequestContext<'req>>
            ) -> Option<::router::http::Response<ResponseBody<'req>>> {
                #conditions

                let req = req.ignore_segments(#matched_segments);
                let result = handler(&req, #(#arguments),*);
                Some(result)
            }

            pub async fn try_match_async<'req>(
                req: &'req ::router::Request<'req, RequestBody<'req>, RequestContext<'req>>
            ) -> Option<::router::http::Response<ResponseBody<'req>>> {
                try_match(req)
            }
        }
    };

    quote! {
        #[allow(non_snake_case)]
        #[doc(hidden)]
        #handler

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        type #handler_request<'req> = #request;

        #[allow(non_camel_case_types)]

        #[doc(hidden)]
        type #handler_response<'req> = #response;

        #[allow(unused_variables)]
        #visibility mod #handler_mod_ident {
            pub use super::#handler_ident as handler;

            pub type RequestBody<'req> = <super::#handler_request<'req> as ::router::Body>::Body;

            pub type RequestContext<'req> = <super::#handler_request<'req> as ::router::Context>::Context;

            pub type ResponseBody<'req> = <super::#handler_response<'req> as ::router::Body>::Body;

            #try_match
        }
    }
}

macro_rules! route_verb {
    ($verb:ident => $value:ident) => {
        #[proc_macro_attribute]
        pub fn $verb(
            segments: proc_macro::TokenStream,
            handler: proc_macro::TokenStream,
        ) -> proc_macro::TokenStream {
            let segments = parse_macro_input!(segments as Segments);
            let handler = parse_macro_input!(handler as syn::ItemFn);

            route_attribute(
                segments,
                handler,
                Some(quote!(::router::http::Method::$value)),
            )
            .into()
        }
    };
}

route_verb!(get => GET);
route_verb!(put => PUT);
route_verb!(post => POST);
route_verb!(delete => DELETE);
route_verb!(head => HEAD);
route_verb!(patch => PATCH);
route_verb!(options => OPTIONS);

#[proc_macro_attribute]
pub fn any(
    segments: proc_macro::TokenStream,
    handler: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let segments = parse_macro_input!(segments as Segments);
    let handler = parse_macro_input!(handler as syn::ItemFn);

    route_attribute(segments, handler, None).into()
}

struct RouterInput {
    asyncness: Option<Token![async]>,
    name: Ident,
    handlers: Punctuated<Path, Token![,]>,
}

impl Parse for RouterInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let asyncness = if input.peek(Token![async]) {
            Some(input.parse()?)
        } else {
            None
        };
        let name = input.parse()?;
        input.parse::<Token![=>]>()?;
        let handlers = Punctuated::parse_terminated(input)?;
        Ok(Self {
            name,
            handlers,
            asyncness,
        })
    }
}

#[proc_macro]
pub fn router(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let RouterInput {
        name,
        handlers,
        asyncness,
    } = parse_macro_input!(input as RouterInput);

    let first = handlers
        .first()
        .expect("Must have at least one handler")
        .clone();

    let handlers = handlers.into_iter();

    let method = if asyncness.is_some() {
        quote!(try_match_async)
    } else {
        quote!(try_match)
    };

    let maybe_await = asyncness.map(|_| quote!(.await));

    quote! {
        struct #name;

        impl #name {
            #asyncness fn route<'req>(
                request: &'req ::router::Request<'req, #first::RequestBody<'req>, #first::RequestContext<'req>>
            ) -> ::core::option::Option<::router::http::Response<#first::ResponseBody<'req>>> {
                #(if let Some(response) = #handlers::#method(&request) #maybe_await {
                    return Some(response);
                })*
                None
            }
        }
    }
    .into()
}
