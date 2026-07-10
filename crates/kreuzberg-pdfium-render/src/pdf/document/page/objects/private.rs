pub(crate) mod internal {

    use crate::bindings::PdfiumLibraryBindings;
    use crate::error::PdfiumError;
    use crate::pdf::document::page::PdfPageObjectOwnership;
    use crate::pdf::document::page::object::PdfPageObject;
    use crate::pdf::document::page::objects::common::{PdfPageObjectIndex, PdfPageObjectsIterator};

    /// Internal crate-specific functionality common to all [PdfPageObjects] collections.
    pub(crate) trait PdfPageObjectsPrivate<'a> {
        /// Returns the ownership hierarchy for this page objects collection.
        fn ownership(&self) -> &PdfPageObjectOwnership;

        /// Returns the [PdfiumLibraryBindings] used by this page objects collection.
        fn bindings(&self) -> &'a dyn PdfiumLibraryBindings;

        /// Internal implementation of [PdfPageObjectsCommon::len()].
        fn len_impl(&self) -> PdfPageObjectIndex;

        /// Internal implementation of [PdfPageObjectsCommon::get()].
        fn get_impl(&self, index: PdfPageObjectIndex) -> Result<PdfPageObject<'a>, PdfiumError>;

        /// Internal implementation of [PdfPageObjectsCommon::iter()].
        fn iter_impl(&'a self) -> PdfPageObjectsIterator<'a>;

        /// Internal implementation of [PdfPageObjectsCommon::add_object()].
        fn add_object_impl(&mut self, object: PdfPageObject<'a>) -> Result<PdfPageObject<'a>, PdfiumError>;

        /// Internal implementation of [PdfPageObjectsCommon::remove_object()].
        fn remove_object_impl(&mut self, object: PdfPageObject<'a>) -> Result<PdfPageObject<'a>, PdfiumError>;
    }
}
