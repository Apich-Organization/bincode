use crate::attribute::ContainerAttributes;
use virtue::parse::Visibility;
use virtue::prelude::*;

pub(crate) struct DeriveZeroCopy {
    pub fields: Option<Fields>,
    pub attributes: ContainerAttributes,
    pub visibility: Visibility,
}

impl DeriveZeroCopy {
    pub fn generate(
        self,
        generator: &mut Generator,
    ) -> Result<()> {
        self.generate_static_size(generator)?;
        self.generate_zerocopy_marker(generator)?;
        self.generate_builder(generator)?;
        Ok(())
    }

    fn generate_static_size(
        &self,
        generator: &mut Generator,
    ) -> Result<()> {
        let crate_name = &self.attributes.crate_name;
        generator
            .impl_for(format!("{}::relative_ptr::StaticSize", crate_name))
            .generate_const("SIZE", "usize")
            .with_value(|builder| {
                builder.push_parsed("core::mem::size_of::<Self>()")?;
                Ok(())
            })?;
        Ok(())
    }

    fn generate_zerocopy_marker(
        &self,
        generator: &mut Generator,
    ) -> Result<()> {
        let crate_name = &self.attributes.crate_name;
        generator
            .impl_for(format!("{}::relative_ptr::ZeroCopy", crate_name))
            .make_unsafe()
            .generate_const("ALIGN", "usize")
            .with_value(|builder| {
                builder.push_parsed("core::mem::align_of::<Self>()")?;
                Ok(())
            })?;
        Ok(())
    }

    fn generate_builder(
        &self,
        generator: &mut Generator,
    ) -> Result<()> {
        let crate_name = &self.attributes.crate_name;
        let target_name = generator.target_name().to_string();
        let builder_name = format!("{}Builder", target_name);

        let endian_type = if let Some((ref e, _)) = self.attributes.endian {
            match e.as_str() {
                | "little" => format!("{}::relative_ptr::LittleEndian", crate_name),
                | "big" => format!("{}::relative_ptr::BigEndian", crate_name),
                | "native" => format!("{}::relative_ptr::NativeEndian", crate_name),
                | _ => {
                    return Err(Error::custom(
                        "Invalid endianness. Expected 'little', 'big', or 'native'",
                    ));
                },
            }
        } else {
            format!("{}::relative_ptr::NativeEndian", crate_name)
        };

        let mut fields_info = Vec::new();
        if let Some(ref fields) = self.fields {
            match fields {
                | Fields::Struct(s) => {
                    for (ident, field) in s {
                        fields_info.push((Some(ident.clone()), field.type_string()));
                    }
                },
                | Fields::Tuple(t) => {
                    for field in t.iter() {
                        fields_info.push((None, field.type_string()));
                    }
                },
            }
        }

        // 1. Define the Builder struct
        {
            let mut builder_struct = generator.generate_struct(&builder_name);
            if self.visibility == Visibility::Pub {
                builder_struct.make_pub();
            }
            for (i, (ident, type_string)) in fields_info.iter().enumerate() {
                let name = ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| format!("field_{}", i));
                builder_struct.add_field(
                    name,
                    format!(
                        "<{} as {}::relative_ptr::ZeroCopyType<{}>>::Builder",
                        type_string, crate_name, endian_type
                    ),
                );
            }
        }

        // 2. Implement ZeroCopyType for the target struct
        let align_value = self.attributes.align.as_ref().map(|a| a.0).unwrap_or(0);
        generator
            .impl_for(format!(
                "{}::relative_ptr::ZeroCopyType<{}>",
                crate_name, endian_type
            ))
            .impl_type("Builder", &builder_name)?;

        // 3. Implement ZeroCopyBuilder for the Builder struct
        {
            let is_tuple = matches!(self.fields, Some(Fields::Tuple(_)));
            let mut impl_for = generator.impl_trait_for_other_type(
                format!(
                    "{}::relative_ptr::ZeroCopyBuilder<{}, {}>",
                    crate_name, endian_type, align_value
                ),
                builder_name,
            );
            impl_for.impl_type("Target", &target_name)?;
            impl_for.generate_fn("build_to_target")
                .with_arg("self", "Self")
                .with_arg("builder", format!("&mut {}::relative_ptr::ZeroBuilder", crate_name))
                .with_arg("offset", "usize")
                .with_return_type("Self::Target")
                .body(|fn_body: &mut StreamBuilder| {
                    fn_body.ident_str(&target_name);
                    let delimiter = if is_tuple { Delimiter::Parenthesis } else { Delimiter::Brace };
                    fn_body.group(delimiter, |struct_body| {
                        for (i, (ident, _)) in fields_info.iter().enumerate() {
                            let builder_field_name = ident.as_ref().map(|i| i.to_string()).unwrap_or_else(|| format!("field_{}", i));
                            if is_tuple {
                                struct_body.push_parsed(format!(
                                    "{}::relative_ptr::ZeroCopyBuilder::<{}, _>::build_to_target(self.{}, builder, offset + core::mem::offset_of!(Self::Target, {})),",
                                    crate_name, endian_type, builder_field_name, i
                                ))?;
                            } else {
                                let field_name = ident.as_ref().map(|i| i.to_string()).expect("Should have ident");
                                struct_body.push_parsed(format!(
                                    "{}: {}::relative_ptr::ZeroCopyBuilder::<{}, _>::build_to_target(self.{}, builder, offset + core::mem::offset_of!(Self::Target, {})),",
                                    field_name, crate_name, endian_type, builder_field_name, field_name
                                ))?;
                            }
                        }
                        Ok(())
                    })?;
                    Ok(())
                })?;
        }

        Ok(())
    }
}
