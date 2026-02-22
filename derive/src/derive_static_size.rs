use crate::attribute::{ContainerAttributes, FieldAttributes};
use virtue::prelude::*;

pub(crate) struct DeriveStaticSize {
    pub fields: Option<Fields>,
    pub variants: Option<Vec<EnumVariant>>,
    pub attributes: ContainerAttributes,
}

/// Build a sum expression string for all fields in a `Fields`.
fn fields_sum_expr(fields: &Fields, crate_name: &str) -> Result<String> {
    let mut parts: Vec<String> = Vec::new();
    match fields {
        Fields::Struct(s) => {
            for (_ident, field) in s {
                let attrs = field
                    .attributes
                    .get_attribute::<FieldAttributes>()?
                    .unwrap_or_default();
                if attrs.static_size_skip {
                    continue;
                }
                if let Some(custom) = attrs.static_size_custom {
                    parts.push(custom);
                } else {
                    parts.push(format!(
                        "<{} as {}::StaticSize>::MAX_SIZE",
                        field.type_string(),
                        crate_name
                    ));
                }
            }
        }
        Fields::Tuple(t) => {
            for field in t {
                let attrs = field
                    .attributes
                    .get_attribute::<FieldAttributes>()?
                    .unwrap_or_default();
                if attrs.static_size_skip {
                    continue;
                }
                if let Some(custom) = attrs.static_size_custom {
                    parts.push(custom);
                } else {
                    parts.push(format!(
                        "<{} as {}::StaticSize>::MAX_SIZE",
                        field.type_string(),
                        crate_name
                    ));
                }
            }
        }
    }
    if parts.is_empty() {
        Ok("0".to_string())
    } else {
        Ok(parts.join(" + "))
    }
}

/// Build a const-compatible "max of N values" expression.
/// Uses nested `if a > b { a } else { b }` which is const-evaluable.
fn const_max_expr(exprs: &[String]) -> String {
    match exprs.len() {
        0 => "0".to_string(),
        1 => exprs[0].clone(),
        _ => {
            // Nest: max(a, max(b, max(c, d)))
            // Using: { let __a = EXPR; let __b = REST; if __a > __b { __a } else { __b } }
            // But in const context we can just use nested ternaries via blocks.
            // Actually, the cleanest approach: use the fact that on nightly const max is available,
            // but for stable we'll build a chain.
            // Let's just chain: a big nested expression isn't great. Let's do sequential.
            //
            // For two: { const A: usize = e1; const B: usize = e2; if A > B { A } else { B } }
            // For more, we fold.
            let mut result = exprs[0].clone();
            for expr in &exprs[1..] {
                result = format!(
                    "{{ const __A: usize = {}; const __B: usize = {}; if __A > __B {{ __A }} else {{ __B }} }}",
                    result, expr
                );
            }
            result
        }
    }
}

/// Compute the StaticSize expression string from fields/variants.
/// This is public so the Decode derive can reuse it for auto-deriving StaticSize.
pub(crate) fn compute_static_size_expr(
    fields: Option<&Fields>,
    variants: Option<&Vec<EnumVariant>>,
    crate_name: &str,
) -> Result<String> {
    if let Some(variants) = variants {
        // Enum: discriminant (max 5 bytes varint for u32) + max(variant sizes)
        if variants.is_empty() {
            Ok("5".to_string())
        } else {
            let mut variant_exprs = Vec::new();
            for variant in variants {
                if let Some(fields) = variant.fields.as_ref() {
                    variant_exprs.push(fields_sum_expr(fields, crate_name)?);
                } else {
                    variant_exprs.push("0".to_string());
                }
            }
            let max_expr = const_max_expr(&variant_exprs);
            Ok(format!("5 + {}", max_expr))
        }
    } else if let Some(fields) = fields {
        fields_sum_expr(fields, crate_name)
    } else {
        Ok("0".to_string())
    }
}

impl DeriveStaticSize {
    pub fn generate(self, generator: &mut Generator) -> Result<()> {
        let crate_name = &self.attributes.crate_name;

        let expr =
            compute_static_size_expr(self.fields.as_ref(), self.variants.as_ref(), crate_name)?;

        generator
            .impl_for(format!("{}::StaticSize", crate_name))
            .modify_generic_constraints(|generics, where_constraints| {
                for g in generics.iter_generics() {
                    where_constraints.push_constraint(g, format!("{}::StaticSize", crate_name))?;
                }
                Ok(())
            })?
            .generate_const("MAX_SIZE", "usize")
            .with_value(|fn_body: &mut StreamBuilder| {
                fn_body.push_parsed(&expr)?;
                Ok(())
            })?;
        Ok(())
    }
}
