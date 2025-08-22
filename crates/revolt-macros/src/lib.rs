use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::{Parse, Parser}, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Expr, ExprArray, FnArg, Ident, ItemFn, LitStr, Pat, Path, Token, Type
};

#[allow(dead_code)]
struct CommandInfo {
    name: LitStr,
    error_ty: Type,
    state: Type,
    children: ExprArray,
}

impl Parse for CommandInfo {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let error_ty = input.parse()?;
        input.parse::<Token![,]>()?;
        let state = input.parse()?;
        input.parse::<Token![,]>()?;
        let children = input.parse()?;

        Ok(Self { name, error_ty, state, children })
    }
}

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let info = parse_macro_input!(attr as CommandInfo);
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let func_name = &func.sig.ident;
    let command_name = info.name.value();
    let error_type = &info.error_ty;
    let state_type = &info.state;
    let parameters = &func.sig.inputs;

    let mut parameter_names = Vec::new();

    let converters = parameters.iter().skip(1).map(|param| {
        let FnArg::Typed(param) = param else { panic!("self parameter") };
        let Pat::Ident(ref ident) = *param.pat else { panic!("not a typed param") };
        let ident = &ident.ident;
        let ty = &param.ty;

        let var_name = format_ident!("__revolt_{ident}");
        parameter_names.push(var_name.clone());

        quote! {
            let __temp = context.words.next().ok_or(::revolt::Error::MissingParameter)?;
            let #var_name = <#ty as Converter<#error_type, #state_type>>::convert(context, __temp).await?;
        }
    });

    // panic!("{:?}", &info.children.elems);

    let children_iter = info.children.elems.iter().map(|child| {
        let Expr::Path(path) = &child else { panic!("Not a path to a command") };

        quote_spanned!(path.span() => {
            let __struct = #path {};
            let __command = __struct.into_command();
            __command
        })
    });

    let children = quote! {
        let mut children = ::std::collections::HashMap::new();
        #({
            let command = #children_iter;
            children.insert(command.name.clone(), command);
        });*
    };

    quote! {
        #func

        #[doc(hidden)]
        #[allow(nonstandard_style)]
        #vis struct #func_name {}

        #[allow(nonstandard_style, clippy::style)]
        impl #func_name {
            pub(crate) fn into_command(self) -> ::revolt::commands::Command<#error_type, #state_type> {
                fn normalized_func<'a>(context: &'a mut ::revolt::commands::Context<'_, #error_type, #state_type>) -> ::revolt::commands::CommandReturn<'a, #error_type> {
                    ::std::boxed::Box::pin(async move {
                        use ::revolt::commands::Converter;

                        #(#converters)*;

                        #func_name(context, #(#parameter_names),*).await
                    })
                }

                #children;

                ::revolt::commands::Command {
                    name: #command_name.to_string(),
                    handle: normalized_func,
                    children
                }
            }
        }
    }.into()
}

#[proc_macro]
pub fn commands(input: TokenStream) -> TokenStream {
    let paths = Punctuated::<Path, Token![,]>::parse_terminated
        .parse(input)
        .unwrap();

    let exprs = paths.iter().map(|path| {
        quote_spanned!(path.span() => {
            let __struct = #path {};
            let __command = __struct.into_command();
            __command
        })
    });

    quote! {
        vec![#(#exprs),*]
    }
    .into()
}
