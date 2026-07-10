pub(crate) mod internal {

    use crate::bindgen::FPDF_ACTION;
    use crate::bindings::PdfiumLibraryBindings;
    use crate::pdf::action::PdfActionCommon;

    /// Internal crate-specific functionality common to all [PdfAction] actions.
    pub(crate) trait PdfActionPrivate<'a>: PdfActionCommon<'a> {
        /// Returns the internal `FPDF_ACTION` handle for this [PdfAction].
        #[allow(dead_code)] // ~keep TODO: remove this allow once handle() is used by action code.
        fn handle(&self) -> &FPDF_ACTION;

        /// Returns the [PdfiumLibraryBindings] used by this [PdfAction].
        fn bindings(&self) -> &dyn PdfiumLibraryBindings;
    }
}
