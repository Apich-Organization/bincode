#![no_main]
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use bincode_next::relative_ptr::DeepValidator;
use bincode_next::relative_ptr::FixedString;
use bincode_next::relative_ptr::RelativeBuilder;
use bincode_next::relative_ptr::ZeroBuilder;
use bincode_next::relative_ptr::ZeroCopyBuilder;
use bincode_next::relative_ptr::ZeroSlice;
use bincode_next::relative_ptr::ZeroStr;
use bincode_next::relative_ptr::ZeroString;
use bincode_next::ZeroCopy;
use libfuzzer_sys::fuzz_target;

#[derive(
    Arbitrary, ZeroCopy, bincode_next::Decode, bincode_next::Encode, PartialEq, Debug, Clone, Copy,
)]
#[repr(C)]
pub struct Inner {
    pub a: u32,
    pub b: bool,
    pub padding: [u8; 3],
}

// Root now uses the enhanced derive for all traits!
#[derive(
    Arbitrary, ZeroCopy, bincode_next::Decode, bincode_next::Encode, PartialEq, Debug, Clone, Copy,
)]
#[repr(C)]
pub struct Root {
    pub id: [u8; 16],
    pub data: ZeroSlice<Inner, 0>,
    pub name: ZeroString<32>,
    pub dynamic_name: bincode_next::relative_ptr::RelativePtr<ZeroStr, 0>,
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    // 1. Generate Arbitrary Input
    let id = match <[u8; 16]>::arbitrary(&mut u) {
        | Ok(v) => v,
        | Err(_) => return,
    };
    let inner_vec = match <Vec<Inner>>::arbitrary(&mut u) {
        | Ok(v) => v,
        | Err(_) => return,
    };
    let name_str = match <String>::arbitrary(&mut u) {
        | Ok(v) => v,
        | Err(_) => return,
    };
    let dyn_name_str = match <String>::arbitrary(&mut u) {
        | Ok(v) => v,
        | Err(_) => return,
    };

    // 2. Build ZeroCopy structure using ZeroBuilder
    let mut builder = ZeroBuilder::new();

    let root_offset = builder.reserve::<Root>();

    // Convert Vec<Inner> to its Builder counterparts (primitives are their own builders)
    // Note: Inner derives ZeroCopy, so its builder is InnerBuilder.
    // Primitives/Arrays derive ZeroCopy but are handled specially.

    let data_slice = inner_vec
        .iter()
        .cloned()
        .map(|i| {
            InnerBuilder {
                a: i.a,
                b: i.b,
                padding: i.padding,
            }
        })
        .collect::<Vec<_>>()
        .build_to_target(&mut builder, root_offset + 16);

    let name_fixed = FixedString::<32>(name_str.clone());
    let name_zero = name_fixed.build_to_target(&mut builder, root_offset + 24);

    let dyn_name_builder = RelativeBuilder::<_, 0>(dyn_name_str.clone());
    let dyn_name_ptr = dyn_name_builder.build_to_target(&mut builder, root_offset + 60);

    let root_val = Root {
        id,
        data: data_slice,
        name: name_zero,
        dynamic_name: dyn_name_ptr,
    };
    builder.write(root_offset, &root_val);

    let buffer = builder.finish();

    // 3. Resolve and Validate
    let resolved_root: &Root = unsafe {
        let ptr = buffer.as_ptr().add(root_offset).cast::<Root>();
        &*ptr
    };

    // Deep Validation (Derived!)
    assert!(
        resolved_root.is_valid_deep(&buffer),
        "Derived deep validation failed on valid structure"
    );

    // Content Validation (PartialEq Derived!)
    assert_eq!(resolved_root.id, id);
    if let Some(slice) = resolved_root.data.get(&buffer) {
        assert_eq!(slice.len(), inner_vec.len());
        for (a, b) in slice.iter().zip(inner_vec.iter()) {
            assert_eq!(a.a, b.a); // Fields are public now!
            assert_eq!(a.b, b.b);
        }
    }

    if let Some(ds_ptr) = resolved_root.dynamic_name.get(&buffer) {
        if let Some(ds) = ds_ptr.get(&buffer) {
            assert_eq!(ds, dyn_name_str);
        }
    }

    // 4. Randomized/Corruption Test
    let arbitrary_root = match <Root>::arbitrary(&mut u) {
        | Ok(v) => v,
        | Err(_) => return,
    };
    // Just ensure it doesn't crash on random data (since Validator should catch it)
    let _ = arbitrary_root.is_valid_deep(&buffer);
});
