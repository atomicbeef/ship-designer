use proc_macro::{TokenStream, Span};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, DataStruct, Fields, Ident, Variant};

#[proc_macro_derive(PacketSerialize)]
pub fn packet_serialize_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    impl_packet_serialize(&ast)
}

fn impl_packet_serialize(ast: &DeriveInput) -> TokenStream {
    match &ast.data {
        Data::Struct(data) => impl_packet_serialize_struct(&ast.ident, &data),
        _ => unimplemented!(),
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
                let index = syn::Index::from(i);
                serialize_calls.extend(quote!(
                    self.#index.serialize(packet);
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
        _ => unimplemented!(),
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
                quote!(#field_name)
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
                    fn deserialize(packet: mut packets::Packet) -> Result<Self, packets::PacketError> {
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

#[proc_macro_derive(IntoPacket, attributes(PacketType))]
pub fn into_packet_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_into_packet(&ast)
}

fn impl_into_packet(ast: &DeriveInput) -> TokenStream {
    let packet_type = ast.attrs.iter()
        .find(|&attr| attr.path().is_ident("PacketType"))
        .expect("PacketType attribute not found")
        .parse_args::<Variant>()
        .unwrap();

    match &ast.data {
        Data::Struct(data) => impl_into_packet_struct(&ast.ident, &data, packet_type),
        _ => unimplemented!(),
    }
}

fn impl_into_packet_struct(name: &Ident, data: &DataStruct, packet_type: Variant) -> TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let mut serialize_calls = quote!();
            
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                serialize_calls.extend(quote!(
                    packets::PacketSerialize::serialize(&value.#field_name, &mut packet);
                ));
            }

            let gen = quote!(
                impl From<&#name> for packets::Packet {
                    fn from(value: &#name) -> Self {
                        let mut packet = packets::Packet::new(packets::PacketType::#packet_type);

                        #serialize_calls

                        packet
                    }
                }
            );

            gen.into()
        },
        Fields::Unnamed(fields) => {
            let mut serialize_calls = quote!();

            for i in 0..fields.unnamed.len() {
                let index = syn::Index::from(i);
                serialize_calls.extend(quote!(
                    packets::PacketSerialize::serialize(&value.#index, &mut packet);
                ));
            }

            let gen = quote!(
                impl From<&#name> for packets::Packet {
                    fn from(value: &#name) -> Self {
                        let mut packet = packets::Packet::new(packets::PacketType::#packet_type);

                        #serialize_calls

                        packet
                    }
                }
            );

            gen.into()
        },
        Fields::Unit => unimplemented!(),
    }
}

#[proc_macro_derive(TryFromPacket)]
pub fn try_from_packet_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impl_try_from_packet(&ast)
}

fn impl_try_from_packet(ast: &DeriveInput) -> TokenStream {
    match &ast.data {
        Data::Struct(data) => impl_try_from_packet_struct(&ast.ident, &data),
        _ => unimplemented!(),
    }
}

fn impl_try_from_packet_struct(name: &Ident, data: &DataStruct) -> TokenStream {
    match &data.fields {
        Fields::Named(fields) => {
            let mut deserialize_calls = quote!();
            
            for field in fields.named.iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_type = &field.ty;
                deserialize_calls.extend(quote!(
                    let #field_name = <#field_type as packets::PacketDeserialize>::deserialize(&mut packet)?;
                ));
            }

            let field_exprs = fields.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote!(#field_name)
            });

            let gen = quote!(
                impl TryFrom<packets::Packet> for #name {
                    type Error = packets::PacketError;

                    fn try_from(mut packet: packets::Packet) -> Result<Self, Self::Error> {
                        #deserialize_calls

                        Ok(#name { #(#field_exprs),* })
                    }
                }
            );

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
                    let #field_name = <#field_type as packets::PacketDeserialize>::deserialize(&mut packet)?;
                ));
            }

            let field_names_iter = field_names.iter();

            let gen = quote! {
                impl TryFrom<packets::Packet> for #name {
                    type Error = packets::PacketError;

                    fn try_from(mut packet: packets::Packet) -> Result<Self, packets::PacketError> {
                        #deserialize_calls

                        Ok(#name(#(#field_names_iter),*))
                    }
                }
            };

            gen.into()
        },
        Fields::Unit => unimplemented!(),
    }
}