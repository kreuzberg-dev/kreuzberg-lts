//! Defines the [PdfPageLinks] struct, exposing functionality related to the
//! links contained within a single `PdfPage`.

use crate::bindgen::{FPDF_DOCUMENT, FPDF_PAGE};
use crate::bindings::PdfiumLibraryBindings;
use crate::error::PdfiumError;
use crate::pdf::link::PdfLink;
use crate::pdf::points::PdfPoints;
use std::ops::{Range, RangeInclusive};
use std::os::raw::c_int;
use std::ptr::null_mut;

/// The zero-based index of a single [PdfLink] inside its containing [PdfPageLinks] collection.
pub type PdfPageLinkIndex = usize;

/// The links contained within a single `PdfPage`.
pub struct PdfPageLinks<'a> {
    page_handle: FPDF_PAGE,
    document_handle: FPDF_DOCUMENT,
    bindings: &'a dyn PdfiumLibraryBindings,
}

impl<'a> PdfPageLinks<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        page_handle: FPDF_PAGE,
        document_handle: FPDF_DOCUMENT,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPageLinks {
            page_handle,
            document_handle,
            bindings,
        }
    }

    /// Returns the [PdfiumLibraryBindings] used by this [PdfPageLinks] collection.
    #[inline]
    pub fn bindings(&self) -> &dyn PdfiumLibraryBindings {
        self.bindings
    }

    /// Returns the number of links in this [PdfPageLinks] collection.
    #[inline]
    pub fn len(&self) -> PdfPageLinkIndex {
        if self.get(0).is_err() {
            return 0;
        }

        if self.get(1).is_err() {
            return 1;
        }

        let mut range_start = 0;
        let mut range_end = 50;

        loop {
            if self.get(range_end).is_err() {
                break;
            } else {
                range_start = range_end;
                range_end *= 2;
            }
        }

        loop {
            let midpoint = range_start + (range_end - range_start) / 2;

            if midpoint == range_start {
                break;
            }

            if self.get(midpoint).is_err() {
                range_end = midpoint;
            } else {
                range_start = midpoint;
            }
        }

        range_end
    }

    /// Returns `true` if this [PdfPageLinks] collection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a Range from `0..(number of links)` for this [PdfPageLinks] collection.
    #[inline]
    pub fn as_range(&self) -> Range<PdfPageLinkIndex> {
        0..self.len()
    }

    /// Returns an inclusive Range from `0..=(number of links - 1)` for this [PdfPageLinks] collection.
    #[inline]
    pub fn as_range_inclusive(&self) -> RangeInclusive<PdfPageLinkIndex> {
        if self.is_empty() { 0..=0 } else { 0..=(self.len() - 1) }
    }

    /// Returns a single [PdfLink] from this [PdfPageLinks] collection.
    pub fn get(&'a self, index: PdfPageLinkIndex) -> Result<PdfLink<'a>, PdfiumError> {
        let mut start_pos = index as c_int;

        let mut handle = null_mut();

        if self.bindings.is_true(
            self.bindings
                .FPDFLink_Enumerate(self.page_handle, &mut start_pos, &mut handle),
        ) && !handle.is_null()
        {
            Ok(PdfLink::from_pdfium(handle, self.document_handle, self.bindings))
        } else {
            Err(PdfiumError::LinkIndexOutOfBounds)
        }
    }

    /// Returns the first [PdfLink] object in this [PdfPageLinks] collection.
    #[inline]
    pub fn first(&'a self) -> Result<PdfLink<'a>, PdfiumError> {
        self.get(0).map_err(|_| PdfiumError::NoPageLinksInCollection)
    }

    /// Returns the last [PdfLink] object in this [PdfPageLinks] collection.
    #[inline]
    pub fn last(&'a self) -> Result<PdfLink<'a>, PdfiumError> {
        self.get(self.len() - 1)
            .map_err(|_| PdfiumError::NoPageLinksInCollection)
    }

    /// Returns the [PdfLink] object at the given position on the containing page, if any.
    pub fn link_at_point(&self, x: PdfPoints, y: PdfPoints) -> Option<PdfLink<'_>> {
        let handle = self
            .bindings
            .FPDFLink_GetLinkAtPoint(self.page_handle, x.value as f64, y.value as f64);

        if handle.is_null() {
            None
        } else {
            Some(PdfLink::from_pdfium(handle, self.document_handle, self.bindings))
        }
    }

    /// Returns an iterator over all the [PdfLink] objects in this [PdfPageLinks] collection.
    #[inline]
    pub fn iter(&self) -> PdfPageLinksIterator<'_> {
        PdfPageLinksIterator::new(self)
    }
}

/// An iterator over all the [PdfLink] objects in a [PdfPageLinksIterator] collection.
pub struct PdfPageLinksIterator<'a> {
    links: &'a PdfPageLinks<'a>,
    next_index: PdfPageLinkIndex,
}

impl<'a> PdfPageLinksIterator<'a> {
    #[inline]
    pub(crate) fn new(links: &'a PdfPageLinks<'a>) -> Self {
        PdfPageLinksIterator { links, next_index: 0 }
    }
}

impl<'a> Iterator for PdfPageLinksIterator<'a> {
    type Item = PdfLink<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.links.get(self.next_index);

        self.next_index += 1;

        next.ok()
    }
}
