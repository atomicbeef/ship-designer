use proc_macro::{TokenStream, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields, Ident};

#[proc_macro_derive(PacketSerialize)]
pub fn packet_serialize_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    impl_packet_serialize(&ast)
}

fn impl_packet_serialize(ast: &DeriveInput) -> TokenStream {
    match &ast.data {
        Data::Struct(data) => impl_packet_serialize_struct(&ast.ident, &data),
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}

fn impl_packet_serialize_struct(name: &Ident, data: &DataStruct) -> TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let mut serialize_calls = quote!();
            
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                serialize_calls.extend(quote!(
                    (&self.#field_name).serialize(packet);
                ));
            }

            let gen = quote! {
                impl packets::PacketSerialize for #name {
                    fn serialize(&self, packet: &mut packets::Packet) {
                        #serialize_calls
                    }
                }
            };

            gen.into()
        },
        Fields::Unnamed(fields) => {
            let mut serialize_calls = quote!();

            for i in 0..fields.unnamed.len() {
                serialize_calls.extend(quote!(
                    self.#i.serialize(packet);
                ));
            }

            let gen = quote! {
                impl packets::PacketSerialize for #name {
                    fn serialize(&self, packet: &mut packets::Packet) {
                        #serialize_calls
                    }
                }
            };

            gen.into()
        },
        Fields::Unit => {
            let gen = quote! {
                impl packets::PacketSerialize for #name {
                    fn serialize(&self, packet: &mut packets::Packet) {}
                }
            };

            gen.into()
        }
    }
}

#[proc_macro_derive(PacketDeserialize)]
pub fn packet_deserialize_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    impl_packet_deserialize(&ast)
}

fn impl_packet_deserialize(ast: &DeriveInput) -> TokenStream {
    match &ast.data {
        Data::Struct(data) => impl_packet_deserialize_struct(&ast.ident, &data),
        Data::Enum(_) => unimplemented!(),
        Data::Union(_) => unimplemented!(),
    }
}

fn impl_packet_deserialize_struct(name: &Ident, data: &DataStruct) -> TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let mut deserialize_calls = quote!();
            
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                deserialize_calls.extend(quote!(
                    let #field_name = <#field_type as packets::PacketDeserialize>::deserialize(packet)?;
                ));
            }

            let field_exprs = fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote!(#field_name: #field_name)
            });

            let gen = quote! {
                impl packets::PacketDeserialize for #name {
                    fn deserialize(packet: &mut packets::Packet) -> Result<Self, packets::PacketError> {
                        #deserialize_calls

                        Ok(#name { #(#field_exprs),* })
                    }
                }
            };

            gen.into()
        },
        Fields::Unnamed(fields) => {
            let mut deserialize_calls = quote!();

            let field_names: Vec<Ident> = fields.unnamed.iter()
                .enumerate()
                .map(|(i, _)| Ident::new(&format!("field_{}", i), Span::call_site().into()))
                .collect();

            for (i, field) in fields.unnamed.iter().enumerate() {
                let field_name = &field_names[i];
                let field_type = &field.ty;
                
                deserialize_calls.extend(quote!(
                    let #field_name = <#field_type as packets::PacketDeserialize>::deserialize(packet)?;
                ));
            }

            let field_names_iter = field_names.iter();

            let gen = quote! {
                impl packets::PacketDeserialize for #name {
                    fn deserialize(packet: &mut packets::Packet) -> Result<Self, packets::PacketError> {
                        #deserialize_calls

                        Ok(#name(#(#field_names_iter),*))
                    }
                }
            };

            gen.into()
        },
        Fields::Unit => {
            let gen = quote! {
                impl packets::PacketDeserialize for #name {
                    fn deserialize(packet: &mut packets::Packet) -> Result<Self, packets::PacketError> {
                        Ok(#name);
                    }
                }
            };

            gen.into()
        }
    }
}