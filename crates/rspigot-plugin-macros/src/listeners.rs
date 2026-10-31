use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Error, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Receiver, ReturnType,
    Signature, Type,
};

const NOT_A_HANDLER: &str = "a #[listeners] block only holds handlers: `async fn name(&self, event: &Event)`; put helpers in another impl block";
const NOT_AN_EVENT: &str = "a handler takes its event as `&PlayerJoin`, `&PlayerLeave` or `&PlayerMove` to observe it, or `&mut PlayerMove` to decide on it";
const NOT_CANCELLABLE: &str = "`&mut` is only allowed for events that can be cancelled: PlayerMove";
const IGNORE_CANCELLED_ON_DECIDER: &str =
    "`#[ignore_cancelled]` only applies to observing handlers (`&Event`)";
const NOT_A_PRIORITY: &str = "`#[priority(...)]` takes one of Lowest, Low, Normal, High, Highest";
const PRIORITIES: [&str; 5] = ["Lowest", "Low", "Normal", "High", "Highest"];
const EVENTS: [(&str, bool); 3] = [
    ("PlayerJoin", false),
    ("PlayerLeave", false),
    ("PlayerMove", true),
];

struct Handler {
    id: u32,
    method: Ident,
    kind: Ident,
    decides: bool,
    priority: Ident,
    ignore_cancelled: bool,
}

#[derive(Default)]
struct Marks {
    priority: Option<Ident>,
    ignore_cancelled: Option<Attribute>,
}

pub(crate) fn expand(args: TokenStream, mut block: ItemImpl) -> TokenStream {
    let mut errors = Vec::new();
    if !args.is_empty() {
        errors.push(Error::new_spanned(&args, "#[listeners] takes no arguments"));
    }
    if let Some((_, path, _)) = &block.trait_ {
        let error = Error::new_spanned(
            path,
            "#[listeners] goes on the plugin's own impl block (`impl MyPlugin { ... }`), not on a trait impl",
        )
        .into_compile_error();
        return quote!(#block #error);
    }
    let handlers: Vec<Handler> = block
        .items
        .iter_mut()
        .zip(1..)
        .filter_map(|(item, id)| match item {
            ImplItem::Fn(method) => handler(method, id, &mut errors),
            other => {
                errors.push(Error::new_spanned(other, NOT_A_HANDLER));
                None
            }
        })
        .collect();
    let specs = handlers.iter().map(spec);
    let arms = handlers.iter().map(arm);
    let errors = errors.iter().map(Error::to_compile_error);
    let plugin = &block.self_ty;
    let (impl_generics, _, where_clause) = block.generics.split_for_impl();
    quote! {
        #block

        impl #impl_generics ::rspigot_plugin::Listeners for #plugin #where_clause {
            fn handlers() -> ::std::vec::Vec<::rspigot_plugin::HandlerSpec> {
                ::std::vec![#(#specs),*]
            }

            async fn dispatch(&self, call: ::rspigot_plugin::Call) -> ::rspigot_plugin::Outcome {
                match call {
                    #(#arms)*
                    _ => ::rspigot_plugin::Outcome::Observed,
                }
            }
        }

        #(#errors)*
    }
}

fn handler(method: &mut ImplItemFn, id: u32, errors: &mut Vec<Error>) -> Option<Handler> {
    let marks = take_marks(&mut method.attrs, errors);
    let (kind, decides) = match event(&method.sig) {
        Ok(found) => found,
        Err(error) => {
            errors.push(error);
            return None;
        }
    };
    if decides && let Some(attr) = &marks.ignore_cancelled {
        errors.push(Error::new_spanned(attr, IGNORE_CANCELLED_ON_DECIDER));
        return None;
    }
    Some(Handler {
        id,
        method: method.sig.ident.clone(),
        kind,
        decides,
        priority: marks
            .priority
            .unwrap_or_else(|| Ident::new("Normal", Span::call_site())),
        ignore_cancelled: marks.ignore_cancelled.is_some(),
    })
}

fn take_marks(attrs: &mut Vec<Attribute>, errors: &mut Vec<Error>) -> Marks {
    let mut marks = Marks::default();
    for attr in std::mem::take(attrs) {
        if attr.path().is_ident("priority") {
            match priority(&attr) {
                Ok(_) if marks.priority.is_some() => errors.push(Error::new_spanned(
                    &attr,
                    "this handler already has a `#[priority(...)]`",
                )),
                Ok(level) => marks.priority = Some(level),
                Err(error) => errors.push(error),
            }
        } else if attr.path().is_ident("ignore_cancelled") {
            if !matches!(attr.meta, Meta::Path(_)) {
                errors.push(Error::new_spanned(
                    &attr,
                    "`#[ignore_cancelled]` takes no arguments",
                ));
            } else if marks.ignore_cancelled.is_some() {
                errors.push(Error::new_spanned(
                    &attr,
                    "this handler already has an `#[ignore_cancelled]`",
                ));
            } else {
                marks.ignore_cancelled = Some(attr);
            }
        } else {
            attrs.push(attr);
        }
    }
    marks
}

fn priority(attr: &Attribute) -> Result<Ident, Error> {
    let level = match &attr.meta {
        Meta::List(list) => syn::parse2::<Ident>(list.tokens.clone()).ok(),
        _ => None,
    };
    level
        .filter(|level| PRIORITIES.iter().any(|name| level == name))
        .ok_or_else(|| Error::new_spanned(attr, NOT_A_PRIORITY))
}

fn event(sig: &Signature) -> Result<(Ident, bool), Error> {
    let plain = sig.asyncness.is_some()
        && sig.constness.is_none()
        && sig.unsafety.is_none()
        && sig.abi.is_none()
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.variadic.is_none()
        && matches!(sig.output, ReturnType::Default);
    let mut inputs = sig.inputs.iter();
    let (
        true,
        Some(FnArg::Receiver(Receiver {
            reference: Some(_),
            mutability: None,
            ..
        })),
        Some(FnArg::Typed(argument)),
        None,
    ) = (plain, inputs.next(), inputs.next(), inputs.next())
    else {
        return Err(Error::new_spanned(sig, NOT_A_HANDLER));
    };
    let Some((name, cancellable, decides)) = event_type(&argument.ty) else {
        return Err(Error::new_spanned(&argument.ty, NOT_AN_EVENT));
    };
    if decides && !cancellable {
        return Err(Error::new_spanned(&argument.ty, NOT_CANCELLABLE));
    }
    Ok((Ident::new(name, Span::call_site()), decides))
}

fn event_type(ty: &Type) -> Option<(&'static str, bool, bool)> {
    let Type::Reference(reference) = ty else {
        return None;
    };
    let Type::Path(path) = &*reference.elem else {
        return None;
    };
    let segment = path
        .path
        .segments
        .last()
        .filter(|segment| path.qself.is_none() && segment.arguments.is_none())?;
    let (name, cancellable) = EVENTS.iter().find(|(name, _)| segment.ident == name)?;
    Some((*name, *cancellable, reference.mutability.is_some()))
}

fn spec(handler: &Handler) -> TokenStream {
    let Handler {
        id,
        kind,
        decides,
        priority,
        ignore_cancelled,
        ..
    } = handler;
    let id = Literal::u32_unsuffixed(*id);
    let access = if *decides {
        quote!(Decide)
    } else {
        quote!(Observe)
    };
    quote! {
        ::rspigot_plugin::HandlerSpec {
            id: #id,
            event: ::rspigot_plugin::EventKind::#kind,
            access: ::rspigot_plugin::Access::#access,
            priority: ::rspigot_plugin::Priority::#priority,
            ignore_cancelled: #ignore_cancelled,
        }
    }
}

fn arm(handler: &Handler) -> TokenStream {
    let Handler {
        id, method, kind, ..
    } = handler;
    let id = Literal::u32_unsuffixed(*id);
    if handler.decides {
        quote! {
            ::rspigot_plugin::Call::#kind { handler: #id, mut event } => {
                self.#method(&mut event).await;
                ::rspigot_plugin::Outcome::Decided { cancel: event.is_cancelled() }
            }
        }
    } else {
        quote! {
            ::rspigot_plugin::Call::#kind { handler: #id, event } => {
                self.#method(&event).await;
                ::rspigot_plugin::Outcome::Observed
            }
        }
    }
}
