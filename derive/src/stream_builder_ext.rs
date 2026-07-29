use virtue::generate::StreamBuilder;
use virtue::prelude::Ident;

pub(crate) trait StreamBuilderExt {
    fn ident_ref(
        &mut self,
        ident: &Ident,
    ) -> &mut Self;
}

impl StreamBuilderExt for StreamBuilder {
    fn ident_ref(
        &mut self,
        ident: &Ident,
    ) -> &mut Self {
        self.ident(ident.clone());
        self
    }
}
