#[doc(hidden)]
#[macro_export]
macro_rules! bincode_error {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $( { $( $(#[$field_meta:meta])* $field:ident : $ftype:ty ),* $(,)? } )? $( ( $( $(#[$tuple_meta:meta])* $ttype:ty ),* $(,)? ) )?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant $( { $( $(#[$field_meta])* $field : $ftype ),* } )? $( ( $( $(#[$tuple_meta])* $ttype ),* ) )?,
            )*
        }

        paste::paste! {
            $(
                $(#[$variant_meta])*
                #[cold]
                #[track_caller]
                #[inline(never)]
                pub(crate) fn [<cold_ $name:snake _ $variant:snake>]<T>(
                    $($($field : $ftype),*)?
                    $($([<arg_ $ttype:snake>] : $ttype),*)?
                ) -> core::result::Result<T, $name> {
                    core::result::Result::Err($name::$variant $( { $($field),* } )? $( ( $( [<arg_ $ttype:snake>] ),* ) )?)
                }
            )*
        }
    };
}
