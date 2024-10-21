use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, FieldsNamed, GenericParam, Generics
};

#[proc_macro_derive(ToCqlData)]
pub fn derive_to_cql(input : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let derive_body = generate_derive_body(&input.data);

    let expanded = quote! {
        impl ToCqlData for #name{
            fn to_cql(self) -> CqlType{
                #derive_body
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}


fn generate_derive_body(data : &Data) -> TokenStream{
    match *data{
        Data::Struct(ref data) => {
            match data.fields{
                Fields::Named(ref fields) =>{
                    let capacity = fields.named.len();
                    let field_itr =
                        fields
                            .named
                            .iter()
                            .map(|f| {
                                let name = &f.ident;
                                quote_spanned! {
                                    f.span() => 
                                        let value = ToCqlData::to_cql(self.#name);
                                        res.insert(stringify!(#name).to_string(), value);
                                }
                            });
                    quote! {
                        let mut res : HashMap<String, CqlType> = HashMap::with_capacity(#capacity);
                        #(#field_itr)*
                        CqlType::Row(res)
                    }
                }
                _ => panic!("unnamed structs not supported")
            }
        }
        _ => panic!("only structs supported")
    }
}

fn get_fields<'a>(data: &'a Data) -> Option<&'a FieldsNamed>{
    match *data{
        Data::Struct(ref data) => {
            if let Fields::Named(ref fields) = data.fields{
                return Some(fields)
            }
            None
        },
        _ => None
    }
}

#[proc_macro_derive(FromCqlData)]
pub fn derive_from_cql(input : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let input: DeriveInput = parse_macro_input!(input);
    let name = input.ident;
    
    let fields :&FieldsNamed = get_fields(&input.data).expect("expected struct with named fields");
    
    let try_from = try_from_struct(fields);
    let from_cql = from_cql_body();
    let expanded = quote! {

        impl TryFrom<&HashMap<String, CqlType>> for #name{
            type Error = ();
            fn try_from(map: &HashMap<String, CqlType>) -> Result<Self, Self::Error>{
                Ok(Self{
                    #try_from
                })
            }
        }

        impl FromCqlData for #name{
            type Error = String;

            fn from_cql(result : &CqlType) -> Result<Self, Self::Error>{
                #from_cql
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn try_from_struct(fields : &FieldsNamed) -> TokenStream{
    let expanded = 
        fields.named
            .iter()
            .map(|f| {
                let name = &f.ident;
                quote_spanned! {
                    f.span() =>
                        #name : {
                            let value = map.get(stringify!(#name)).ok_or(())?;
                            FromCqlData::from_cql(value)?
                        },
                }
            });
    quote!{
        #(#expanded)*
    }
}

fn from_cql_body() -> TokenStream{
    quote! {
        match result {
            CqlType::Row(r) => {
                r.try_into().map_err(|_e| "type mismatch".to_string())
            },
            _ => Err("only expecting row variant".to_string())
        }
    }
}

// fn generic_field_code_setter<F>(fields : &FieldsNamed , content : F) -> TokenStream
// where
//     F: FnOnce() -> TokenStream
// {
//     let res = fields.named.iter()
//         .map(|f| {
//             let name = &f.ident;
//             content()
//         });
    
//     quote! {
//         #(#res)*
//     }
// }

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
use syn::{ExprAssign, Ident, Result, Token, Expr, punctuated::*, Attribute, Expr::*};
use quote::ToTokens;
use syn::parse::ParseStream;
use std::collections::HashSet;
use proc_macro2::Span;
use std::collections::HashMap;

#[derive(Default)]
struct Args{
    primary_key : Option<Vec<String>>,
    clustering_keys: Option<Vec<String>>,
    table_name: Option<String>,
    keyspace : Option<String>,
}

enum SinInputError{
    E01,
    E02,
    E03,
    E04
}

// impl TryFrom<Vec<ExprAssign>> for Args{
//     type Error = SinInputError;

//     fn try_from(value: Vec<ExprAssign>) -> std::result::Result<Self, Self::Error> {
//         let map = value.into_iter()
//             .map(|exp | {
//                 let left = get_left_path(exp.left)?;
//                 if left == 
//                 let right = get_right_info(exp);
//                 (left, right)
//             })
//             .collect::<HashMap<Ident, >>()
//         Ok(Self::default())
//     }
// }

// fn get_left_path(exp : Box<Expr>) -> std::result::Result<Ident, SinInputError>{
//     match *exp{
//         Path(path) => {
//             path.path.get_ident().map(|i| i.clone()).ok_or(())
//         },
//         _ => Err(()),
//     }
// }

// fn get_array_exp(exp : Box<Expr>) -> std::result::Result<(),SinInputError>{
//     match *exp{
//         Expr::Array(arr) =>,
//         _ => Err(())
//     }
// }
// }

use syn::parse::Parse;
/// #[read_functions(Table{pkey = (), skey = [(), ()], table_name = name, keyspace = name})]
impl Parse for Args{
    fn parse(input: ParseStream) -> Result<Self>{
        let mut primary_key = None;
        let mut clustering_keys = None; 
        let mut table_name= None;
        let mut keyspace= None;

        while !input.is_empty(){
            let key: syn::Ident = input.parse()?;
            let _eq = input.parse::<syn::Token![=]>()?;

            match key.to_string().as_str(){
                "table" => {
                    let value : syn::Expr = input.parse()?;
                    table_name = Some(value.to_token_stream().to_string());
                },
                "partition_key" =>{
                    let value : Vec<String> =
                        input
                            .parse::<syn::ExprArray>()?
                            .elems
                            .into_iter()
                            .map(|e| e.to_token_stream().to_string())
                            .collect();
                    
                    primary_key = Some(value);
                },
                "clustering_key" =>{
                    let value : Vec<String> =
                        input
                            .parse::<syn::ExprArray>()?
                            .elems
                            .into_iter()
                            .map(|e| e.to_token_stream().to_string())
                            .collect();
                    
                    clustering_keys = Some(value);
                    
                },
                "keyspace" =>{
                    let value : syn::Expr = input.parse()?;
                    keyspace = Some(value.to_token_stream().to_string());
                }
                _ => {}
            }
            
            if !input.is_empty(){
                input.parse::<syn::Token![,]>()?;
            }
        }
        
        Ok(Self{
            primary_key,
            clustering_keys,
            table_name,
            keyspace
        })

    }
}

// #[proc_macro_attribute]
// pub fn read_functions(attrs: proc_macro::TokenStream, input : proc_macro::TokenStream) -> proc_macro::TokenStream{
//     let args = parse_macro_input!(attrs as Args);
    
//     let name = input.ident;

//     let find_functions = generate_find_functions(&input.data, &args);
//     let create_function = generate_create_body(&input.data, args.table_name, args.keyspace);

//     let expanded = quote! {
//         impl #name{
            
//         }
//     };
    
//     proc_macro::TokenStream::from(expanded)
// }
      
// fn generate_find_functions(data: &Data, args: &Args) -> proc_macro::TokenStream{

// }

#[proc_macro_attribute]
pub fn nosql(attrs: proc_macro::TokenStream, minput : proc_macro::TokenStream) -> proc_macro::TokenStream{
    let args : Args = parse_macro_input!(attrs);
    //let cinput = minput.clone();
    let input: DeriveInput = parse_macro_input!(minput);
    let table = args.table_name.expect("table name expected");
    let keyspace = args.keyspace.expect("keyspace expected");
    let name = input.ident.clone();
    
    let pre_req = quote!{
        #[derive(sin::ToCqlData, sin::FromCqlData)]
    };

    let res = quote!{
        impl NoSql for #name {
            fn table_name() -> &'static str{
                #table
            }

            fn keyspace() -> &'static str{
                #keyspace
            }
        }

    };

    proc_macro::TokenStream::from(quote! {
        #pre_req
        #input
        #res
    })

}