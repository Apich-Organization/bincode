use virtue::prelude::Ident;
use virtue::generate::StreamBuilder;

pub(crate) trait StreamBuilderExt {
    fn ident_ref(&mut self, ident: &Ident) -> &mut Self;
}

impl StreamBuilderExt for StreamBuilder {
    fn ident_ref(&mut self, ident: &Ident) -> &mut Self {
        self.ident(ident.clone());
        self
    }
}
