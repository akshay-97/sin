use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics,
};


#[proc_macro_derive(Gen)]
pub fn derive_gen(input : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;


    let generics = gen_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let bind_body = generate_body(&input.data);

    //panic!("{}", bind_body.to_string());
    let expanded = quote! {
        impl #impl_generics Gen for #name #ty_generics #where_clause{
            fn bind_insert_statement(&self, s : &mut Statement){
                #bind_body
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn generate_body(data : &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
             match data.fields {
                Fields::Named(ref fields) => {
                    let field_itr =
                        fields.named
                        .iter()
                        .map(|f| {
                            let name = &f.ident;
                            quote_spanned! {f.span() => 
                                let value = BindType::bind_the_type(&self.#name);
                                s.bind_by_name(stringify!(#name), value);
                            }
                        });
                    quote! {
                        #(#field_itr)*
                    }
                }
                _ => unimplemented!()
            }
        },
        _ => unimplemented!()
    }
}

fn gen_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param{
            type_param.bounds.push(parse_quote!(BindType));
        }
    }
    generics
}
use syn::{ExprAssign, Ident, Result, Token, Expr, punctuated::*, Attribute};
use syn::parse::ParseStream;
use std::collections::HashSet;
use proc_macro2::Span;



trait Walk{
    fn walk_and_parse<Output>(&mut self) -> Result<Output, ()>;    
}

#[derive(Default)]
struct Args{
    primary_key : HashSet<Ident>,
    secondary_keys: Vec<HashSet<Ident>>,
    table_name: String,
    keyspace : String,
}

use syn::parse::Parse;
/// #[read_functions(Table{pkey = (), skey = [(), ()], table_name = name, keyspace = name})]
impl Parse for Args{
    fn parse(input: ParseStream) -> Result<Self>{
        let fields = Punctuated::<ExprAssign, Token![,]>::parse_terminated(input)?;
        let pkey_ident = Ident::new("primary_key", Span::call_site());
        let skey_ident = Ident::new("secondary_key", Span::call_site());
        let tname = Ident::new("table_name", Span::call_site());
        let keysp = Ident::new("keyspace", Span::call_site());

        let attr : Vec<ExprAssign> = fields.into_iter().collect();
    
        let out  = Self{
            primary_key : attr.parse()?,
            secondary_keys : attr.parse()?,
            table_name : attr.parse()?,
            keyspace : attr.parse()?,
        };
        return Ok(out);
        panic!("{:?}", attr);
        Ok(Self::default())

    }
}

#[proc_macro_attribute]
pub fn read_functions(attrs: proc_macro::TokenStream, input : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let args = parse_macro_input!(attrs as Args);
    panic!("{}", input.to_string());
    input
}
      
        
