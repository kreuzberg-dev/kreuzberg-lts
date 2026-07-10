use crate::enums::{TessPageIteratorLevel, TessPageSegMode};
use crate::error::{Result, TesseractError};
use crate::page_iterator::{TessBaseAPIGetIterator, TessPageIteratorDelete};
use crate::result_iterator::TessResultIteratorDelete;
use crate::{PageIterator, ResultIterator};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_float, c_int, c_void};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Batch bounding box results from Tesseract layout analysis.
///
/// Holds all bounding boxes returned in a single FFI call, along with optional
/// block and paragraph IDs when available (e.g., from `get_textlines`).
///
/// Each box is represented as `(x, y, width, height)` in image coordinates.
///
/// # Examples
///
/// ```rust,no_run
/// # use kreuzberg_tesseract::TesseractAPI;
/// # let api = TesseractAPI::new().unwrap();
/// # api.init("/tessdata", "eng").unwrap();
/// # api.set_image(&[], 1, 1, 1, 1).unwrap();
/// let words = api.get_words().unwrap();
/// for i in 0..words.len() {
///     if let Some((x, y, w, h)) = words.get(i) {
///         println!("Word at ({x},{y}) size {w}x{h}");
///     }
/// }
/// ```
pub struct BoundingBoxArray {
    boxes: Vec<(i32, i32, i32, i32)>,
    block_ids: Option<Vec<i32>>,
    para_ids: Option<Vec<i32>>,
}

impl BoundingBoxArray {
    /// Returns the number of bounding boxes in the array.
    pub fn len(&self) -> usize {
        self.boxes.len()
    }

    /// Returns `true` if the array contains no bounding boxes.
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    /// Returns the bounding box at `index` as `(x, y, width, height)`, or `None` if out of range.
    pub fn get(&self, index: usize) -> Option<(i32, i32, i32, i32)> {
        self.boxes.get(index).copied()
    }

    /// Returns the block ID for the box at `index`, if block IDs were captured.
    pub fn block_id(&self, index: usize) -> Option<i32> {
        self.block_ids.as_ref()?.get(index).copied()
    }

    /// Returns the paragraph ID for the box at `index`, if paragraph IDs were captured.
    pub fn para_id(&self, index: usize) -> Option<i32> {
        self.para_ids.as_ref()?.get(index).copied()
    }

    /// Returns an iterator over all `(x, y, width, height)` tuples.
    pub fn iter(&self) -> impl Iterator<Item = &(i32, i32, i32, i32)> {
        self.boxes.iter()
    }
}

#[derive(Clone)]
pub struct TesseractConfiguration {
    datapath: String,
    language: String,
    variables: HashMap<String, String>,
}

/// Main interface to the Tesseract OCR engine.
#[cfg(any(feature = "build-tesseract", feature = "build-tesseract-wasm"))]
pub struct TesseractAPI {
    /// Handle to the Tesseract engine.
    pub handle: Arc<Mutex<*mut c_void>>,
    config: Arc<Mutex<TesseractConfiguration>>,
}

unsafe impl Send for TesseractAPI {}
unsafe impl Sync for TesseractAPI {}

#[cfg(any(feature = "build-tesseract", feature = "build-tesseract-wasm"))]
impl TesseractAPI {
    /// Creates a new instance of the Tesseract API.
    ///
    /// # Returns
    ///
    /// Returns `Ok(TesseractAPI)` on success, or `Err(TesseractError::NullPointerError)`
    /// if the underlying C library fails to allocate the engine handle.
    ///
    /// # Errors
    ///
    /// Returns [`TesseractError::NullPointerError`] when `TessBaseAPICreate` returns a
    /// null pointer, which indicates an allocation failure in the Tesseract C library.
    pub fn new() -> Result<Self> {
        // SAFETY: TessBaseAPICreate() is a C FFI function that allocates and initializes
        let handle = unsafe { TessBaseAPICreate() };
        if handle.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        Ok(TesseractAPI {
            handle: Arc::new(Mutex::new(handle)),
            config: Arc::new(Mutex::new(TesseractConfiguration {
                datapath: String::new(),
                language: String::new(),
                variables: HashMap::new(),
            })),
        })
    }

    /// Gets the version of the Tesseract engine.
    ///
    /// # Returns
    ///
    /// Returns the version of the Tesseract engine as a string.
    pub fn version() -> String {
        // SAFETY: TessVersion() returns a pointer to a valid, null-terminated C string that is
        let version = unsafe { TessVersion() };
        unsafe { CStr::from_ptr(version) }.to_string_lossy().into_owned()
    }

    /// Initializes the Tesseract engine with the specified datapath and language.
    ///
    /// # Arguments
    ///
    /// * `datapath` - Path to the directory containing Tesseract data files.
    /// * `language` - Language code (e.g., "eng" for English).
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization is successful, otherwise returns an error.
    pub fn init<P: AsRef<Path>>(&self, datapath: P, language: &str) -> Result<()> {
        let datapath_str = datapath
            .as_ref()
            .to_str()
            .ok_or(TesseractError::InvalidParameterError)?
            .to_owned();
        let language_str = language.to_owned();

        {
            let mut config = self.config.lock().map_err(|_| TesseractError::MutexLockError)?;
            config.datapath = datapath_str.clone();
            config.language = language_str.clone();
        }

        let datapath = CString::new(datapath_str).map_err(|_| TesseractError::NullByteInString)?;
        let language = CString::new(language_str).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIInit3() is a C FFI function that initializes the Tesseract engine.
        let result = unsafe { TessBaseAPIInit3(*handle, datapath.as_ptr(), language.as_ptr()) };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    /// Gets the confidence values for all recognized words.
    ///
    /// # Returns
    ///
    /// Returns a vector of confidence values (0-100) for each recognized word.
    pub fn get_word_confidences(&self) -> Result<Vec<i32>> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;

        // SAFETY: TessBaseAPIAllWordConfidences() returns a pointer to a C array of i32 values
        let confidences_ptr = unsafe { TessBaseAPIAllWordConfidences(*handle) };
        if confidences_ptr.is_null() {
            return Ok(Vec::new());
        }
        let mut confidences = Vec::new();
        let mut i = 0;
        // SAFETY: We iterate through the array using pointer arithmetic (offset()).
        while unsafe { *confidences_ptr.offset(i) } != -1 {
            confidences.push(unsafe { *confidences_ptr.offset(i) });
            i += 1;
        }
        // SAFETY: TessDeleteIntArray() deallocates the array returned by TessBaseAPIAllWordConfidences():
        unsafe { TessDeleteIntArray(confidences_ptr) };
        Ok(confidences)
    }

    /// Gets the mean text confidence.
    ///
    /// # Returns
    ///
    /// Returns the mean text confidence as an integer.
    pub fn mean_text_conf(&self) -> Result<i32> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIMeanTextConf() is a simple FFI call that returns a computed i32 value.
        Ok(unsafe { TessBaseAPIMeanTextConf(*handle) })
    }

    /// Sets a Tesseract variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable.
    /// * `value` - Value to set.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the variable is successful, otherwise returns an error.
    pub fn set_variable(&self, name: &str, value: &str) -> Result<()> {
        {
            let mut config = self.config.lock().map_err(|_| TesseractError::MutexLockError)?;
            config.variables.insert(name.to_owned(), value.to_owned());
        }

        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let value = CString::new(value).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetVariable() is safe because:
        let result = unsafe { TessBaseAPISetVariable(*handle, name.as_ptr(), value.as_ptr()) };
        if result != 1 {
            Err(TesseractError::SetVariableError)
        } else {
            Ok(())
        }
    }

    /// Gets a string variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable.
    ///
    /// # Returns
    ///
    /// Returns the value of the variable as a string.
    pub fn get_string_variable(&self, name: &str) -> Result<String> {
        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetStringVariable() returns a pointer to a C string stored in
        let value_ptr = unsafe { TessBaseAPIGetStringVariable(*handle, name.as_ptr()) };
        if value_ptr.is_null() {
            return Err(TesseractError::GetVariableError);
        }
        // SAFETY: We've verified value_ptr is non-null. CStr::from_ptr() is safe because:
        let c_str = unsafe { CStr::from_ptr(value_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    /// Gets an integer variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable.
    ///
    /// # Returns
    ///
    /// Returns the value of the variable as an integer.
    pub fn get_int_variable(&self, name: &str) -> Result<i32> {
        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetIntVariable() is safe because:
        Ok(unsafe { TessBaseAPIGetIntVariable(*handle, name.as_ptr()) })
    }

    /// Gets a boolean variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable.
    ///
    /// # Returns
    ///
    /// Returns the value of the variable as a boolean.
    pub fn get_bool_variable(&self, name: &str) -> Result<bool> {
        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetBoolVariable() is safe because:
        Ok(unsafe { TessBaseAPIGetBoolVariable(*handle, name.as_ptr()) } != 0)
    }

    /// Gets a double variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable.
    ///
    /// # Returns
    ///
    /// Returns the value of the variable as a double.
    pub fn get_double_variable(&self, name: &str) -> Result<f64> {
        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetDoubleVariable() is safe because:
        Ok(unsafe { TessBaseAPIGetDoubleVariable(*handle, name.as_ptr()) })
    }

    /// Sets the page segmentation mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - Page segmentation mode.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the page segmentation mode is successful, otherwise returns an error.
    pub fn set_page_seg_mode(&self, mode: TessPageSegMode) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetPageSegMode() is safe because:
        unsafe { TessBaseAPISetPageSegMode(*handle, mode as c_int) };
        Ok(())
    }

    /// Gets the page segmentation mode.
    ///
    /// # Returns
    ///
    /// Returns the page segmentation mode.
    pub fn get_page_seg_mode(&self) -> Result<TessPageSegMode> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetPageSegMode() is safe because:
        let mode = unsafe { TessBaseAPIGetPageSegMode(*handle) };
        TessPageSegMode::try_from_int(mode).ok_or(TesseractError::InvalidEnumValue(mode))
    }

    /// Recognizes the text in the current image.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if recognition is successful, otherwise returns an error.
    pub fn recognize(&self) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIRecognize() is safe because:
        let result = unsafe { TessBaseAPIRecognize(*handle, std::ptr::null_mut()) };
        if result != 0 {
            Err(TesseractError::OcrError)
        } else {
            Ok(())
        }
    }

    /// Gets the HOCR text for the specified page.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number.
    ///
    /// # Returns
    ///
    /// Returns the HOCR text for the specified page as a string.
    pub fn get_hocr_text(&self, page: i32) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetHOCRText() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetHOCRText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        // SAFETY: We've verified text_ptr is non-null. CStr::from_ptr() is safe because:
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        // SAFETY: TessDeleteText() correctly frees the memory allocated by TessBaseAPIGetHOCRText().
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    /// Gets the ALTO text for the specified page.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number.
    ///
    /// # Returns
    ///
    /// Returns the ALTO text for the specified page as a string.
    pub fn get_alto_text(&self, page: i32) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetAltoText() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetAltoText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        // SAFETY: We've verified text_ptr is non-null. CStr::from_ptr() and TessDeleteText()
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        // SAFETY: TessDeleteText() must be called to free the allocated string
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    /// Gets the TSV text for the specified page.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number.
    ///
    /// # Returns
    ///
    /// Returns the TSV text for the specified page as a string.
    pub fn get_tsv_text(&self, page: i32) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetTsvText() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetTsvText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        // SAFETY: CStr::from_ptr() is safe because text_ptr is non-null and points to
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        // SAFETY: TessDeleteText() frees the memory allocated by TessBaseAPIGetTsvText()
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    /// Sets the input name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the input.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the input name is successful, otherwise returns an error.
    pub fn set_input_name(&self, name: &str) -> Result<()> {
        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetInputName() is safe because:
        unsafe { TessBaseAPISetInputName(*handle, name.as_ptr()) };
        Ok(())
    }

    /// Gets the input name.
    ///
    /// # Returns
    ///
    /// Returns the input name as a string.
    pub fn get_input_name(&self) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetInputName() returns a pointer to a C string in Tesseract's memory.
        let name_ptr = unsafe { TessBaseAPIGetInputName(*handle) };
        if name_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        // SAFETY: We've verified name_ptr is non-null. CStr::from_ptr() is safe because:
        let c_str = unsafe { CStr::from_ptr(name_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    /// Gets the data path.
    ///
    /// # Returns
    ///
    /// Returns the data path as a string.
    pub fn get_datapath(&self) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetDatapath() returns a pointer to a C string in Tesseract's memory.
        let path_ptr = unsafe { TessBaseAPIGetDatapath(*handle) };
        if path_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        // SAFETY: CStr::from_ptr() is safe because we've verified path_ptr is non-null
        let c_str = unsafe { CStr::from_ptr(path_ptr) };
        Ok(c_str.to_str()?.to_owned())
    }

    /// Gets the source Y resolution.
    ///
    /// # Returns
    ///
    /// Returns the source Y resolution as an integer.
    pub fn get_source_y_resolution(&self) -> Result<i32> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetSourceYResolution() is safe because:
        Ok(unsafe { TessBaseAPIGetSourceYResolution(*handle) })
    }

    /// Gets the thresholded image.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the thresholded image.
    pub fn get_thresholded_image(&self) -> Result<*mut c_void> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetThresholdedImage() returns a pointer to a Pix structure.
        let pix = unsafe { TessBaseAPIGetThresholdedImage(*handle) };
        if pix.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(pix)
        }
    }

    /// Gets the box text for the specified page.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number.
    ///
    /// # Returns
    ///
    /// Returns the box text for the specified page as a string.
    pub fn get_box_text(&self, page: i32) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetBoxText() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetBoxText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        // SAFETY: CStr::from_ptr() is safe because text_ptr is non-null and points to
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        // SAFETY: TessDeleteText() frees the memory allocated by TessBaseAPIGetBoxText()
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    /// Gets the LSTM box text for the specified page.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number.
    ///
    /// # Returns
    ///
    /// Returns the LSTM box text for the specified page as a string.
    pub fn get_lstm_box_text(&self, page: i32) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetLSTMBoxText() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetLSTMBoxText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        // SAFETY: CStr::from_ptr() is safe because text_ptr is non-null and points to
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        // SAFETY: TessDeleteText() frees the memory allocated by TessBaseAPIGetLSTMBoxText()
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    /// Gets the word str box text for the specified page.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number.
    ///
    /// # Returns
    ///
    /// Returns the word str box text for the specified page as a string.
    pub fn get_word_str_box_text(&self, page: i32) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetWordStrBoxText() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetWordStrBoxText(*handle, page) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        // SAFETY: CStr::from_ptr() is safe because text_ptr is non-null and points to
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        // SAFETY: TessDeleteText() frees the memory allocated by TessBaseAPIGetWordStrBoxText()
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    /// Gets the UNLV text.
    ///
    /// # Returns
    ///
    /// Returns the UNLV text as a string.
    pub fn get_unlv_text(&self) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetUNLVText() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetUNLVText(*handle) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        // SAFETY: CStr::from_ptr() is safe because text_ptr is non-null and points to
        let c_str = unsafe { CStr::from_ptr(text_ptr) };
        let result = c_str.to_str()?.to_owned();
        // SAFETY: TessDeleteText() frees the memory allocated by TessBaseAPIGetUNLVText()
        unsafe { TessDeleteText(text_ptr) };
        Ok(result)
    }

    /// Gets all word confidences.
    ///
    /// # Returns
    ///
    /// Returns a vector of all word confidences.
    pub fn all_word_confidences(&self) -> Result<Vec<i32>> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIAllWordConfidences() returns a pointer to a C array of i32 values
        let confidences_ptr = unsafe { TessBaseAPIAllWordConfidences(*handle) };
        if confidences_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }
        let mut confidences = Vec::new();
        let mut i = 0;
        // SAFETY: We iterate through the array using pointer arithmetic (offset()).
        while unsafe { *confidences_ptr.offset(i) } != -1 {
            confidences.push(unsafe { *confidences_ptr.offset(i) });
            i += 1;
        }
        // SAFETY: TessDeleteIntArray() deallocates the array returned by TessBaseAPIAllWordConfidences():
        unsafe { TessDeleteIntArray(confidences_ptr) };
        Ok(confidences)
    }

    /// Adapts to the word string.
    ///
    /// # Arguments
    ///
    /// * `mode` - Mode.
    /// * `wordstr` - Word string.
    ///
    /// # Returns
    ///
    /// Returns `true` if adaptation is successful, otherwise returns `false`.
    pub fn adapt_to_word_str(&self, mode: i32, wordstr: &str) -> Result<bool> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        let wordstr = CString::new(wordstr).map_err(|_| TesseractError::NullByteInString)?;
        // SAFETY: TessBaseAPIAdaptToWordStr() is safe because:
        let result = unsafe { TessBaseAPIAdaptToWordStr(*handle, mode, wordstr.as_ptr()) };
        Ok(result != 0)
    }

    /// Detects the orientation and script.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing the orientation in degrees, the orientation confidence, the script name, and the script confidence.
    pub fn detect_os(&self) -> Result<(i32, f32, String, f32)> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        let mut orient_deg = 0;
        let mut orient_conf = 0.0;
        let mut script_name_ptr = std::ptr::null_mut();
        let mut script_conf = 0.0;
        // SAFETY: TessBaseAPIDetectOrientationScript() is safe because:
        let result = unsafe {
            TessBaseAPIDetectOrientationScript(
                *handle,
                &mut orient_deg,
                &mut orient_conf,
                &mut script_name_ptr,
                &mut script_conf,
            )
        };
        if result == 0 {
            return Err(TesseractError::OcrError);
        }
        let script_name = if !script_name_ptr.is_null() {
            // SAFETY: script_name_ptr is non-null and points to a valid null-terminated C string
            let c_str = unsafe { CStr::from_ptr(script_name_ptr) };
            let result = c_str.to_str()?.to_owned();
            // SAFETY: TessDeleteText() must be called exactly once to free the string allocated
            unsafe { TessDeleteText(script_name_ptr) };
            result
        } else {
            String::new()
        };
        Ok((orient_deg, orient_conf, script_name, script_conf))
    }

    /// Sets the minimum orientation margin.
    ///
    /// # Arguments
    ///
    /// * `margin` - Minimum orientation margin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the minimum orientation margin is successful, otherwise returns an error.
    pub fn set_min_orientation_margin(&self, margin: f64) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetMinOrientationMargin() is safe because:
        unsafe { TessBaseAPISetMinOrientationMargin(*handle, margin) };
        Ok(())
    }

    /// Gets the page iterator.
    ///
    /// # Returns
    ///
    /// Returns a `PageIterator` object.
    pub fn get_page_iterator(&self) -> Result<PageIterator> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetIterator() returns a pointer to a PageIterator structure.
        let iterator = unsafe { TessBaseAPIGetIterator(*handle) };
        if iterator.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        Ok(PageIterator::new(iterator))
    }

    /// Sets the input image.
    ///
    /// # Arguments
    ///
    /// * `pix` - Pointer to the input image.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the input image is successful, otherwise returns an error.
    pub fn set_input_image(&self, pix: *mut c_void) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetInputImage() is safe because:
        unsafe { TessBaseAPISetInputImage(*handle, pix) };
        Ok(())
    }

    /// Gets the input image.
    ///
    /// # Returns
    ///
    /// Returns a pointer to the input image.
    pub fn get_input_image(&self) -> Result<*mut c_void> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetInputImage() returns a pointer to the input Pix.
        let pix = unsafe { TessBaseAPIGetInputImage(*handle) };
        if pix.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(pix)
        }
    }

    /// Sets the output name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the output.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the output name is successful, otherwise returns an error.
    pub fn set_output_name(&self, name: &str) -> Result<()> {
        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetOutputName() is safe because:
        unsafe { TessBaseAPISetOutputName(*handle, name.as_ptr()) };
        Ok(())
    }

    /// Sets the debug variable.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable.
    /// * `value` - Value of the variable.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the debug variable is successful, otherwise returns an error.
    pub fn set_debug_variable(&self, name: &str, value: &str) -> Result<()> {
        let name = CString::new(name).map_err(|_| TesseractError::NullByteInString)?;
        let value = CString::new(value).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetDebugVariable() is safe because:
        let result = unsafe { TessBaseAPISetDebugVariable(*handle, name.as_ptr(), value.as_ptr()) };
        if result != 1 {
            Err(TesseractError::SetVariableError)
        } else {
            Ok(())
        }
    }

    /// Prints the variables to a file.
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the file to print the variables to.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if printing the variables to the file is successful, otherwise returns an error.
    pub fn print_variables_to_file(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIPrintVariablesToFile() is safe because:
        let result = unsafe { TessBaseAPIPrintVariablesToFile(*handle, filename.as_ptr()) };
        if result != 0 {
            Err(TesseractError::IoError)
        } else {
            Ok(())
        }
    }

    /// Initializes for analysing a page.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization is successful, otherwise returns an error.
    pub fn init_for_analyse_page(&self) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIInitForAnalysePage() is safe because:
        unsafe { TessBaseAPIInitForAnalysePage(*handle) };
        Ok(())
    }
    /// Reads the configuration file.
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the configuration file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if reading the configuration file is successful, otherwise returns an error.
    pub fn read_config_file(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIReadConfigFile() is safe because:
        unsafe { TessBaseAPIReadConfigFile(*handle, filename.as_ptr()) };
        Ok(())
    }

    /// Reads the debug configuration file.
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the debug configuration file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if reading the debug configuration file is successful, otherwise returns an error.
    pub fn read_debug_config_file(&self, filename: &str) -> Result<()> {
        let filename = CString::new(filename).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIReadDebugConfigFile() is safe because:
        unsafe { TessBaseAPIReadDebugConfigFile(*handle, filename.as_ptr()) };
        Ok(())
    }

    /// Gets the thresholded image scale factor.
    ///
    /// # Returns
    ///
    /// Returns the thresholded image scale factor as an integer.
    pub fn get_thresholded_image_scale_factor(&self) -> Result<i32> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetThresholdedImageScaleFactor() is safe because:
        Ok(unsafe { TessBaseAPIGetThresholdedImageScaleFactor(*handle) })
    }

    /// Processes the pages.
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the file to process.
    /// * `retry_config` - Retry configuration.
    /// * `timeout_millisec` - Timeout in milliseconds.
    ///
    /// # Returns
    ///
    /// Returns the processed text as a string.
    pub fn process_pages(&self, filename: &str, retry_config: Option<&str>, timeout_millisec: i32) -> Result<String> {
        let filename = CString::new(filename).map_err(|_| TesseractError::NullByteInString)?;
        let retry_config_cstr = retry_config
            .map(|s| CString::new(s).map_err(|_| TesseractError::NullByteInString))
            .transpose()?;
        let retry_ptr = retry_config_cstr.as_ref().map_or(std::ptr::null(), |c| c.as_ptr());
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIProcessPages() is safe because:
        let result = unsafe {
            TessBaseAPIProcessPages(
                *handle,
                filename.as_ptr(),
                retry_ptr,
                timeout_millisec,
                std::ptr::null_mut(),
            )
        };
        if result.is_null() {
            Err(TesseractError::ProcessPagesError)
        } else {
            // SAFETY: We've verified result is non-null. CStr::from_ptr() is safe because:
            let c_str = unsafe { CStr::from_ptr(result) };
            let output = c_str.to_str()?.to_owned();
            // SAFETY: TessDeleteText() must be called exactly once to free the string
            unsafe { TessDeleteText(result) };
            Ok(output)
        }
    }

    /// Gets the initial languages as a string.
    ///
    /// # Returns
    ///
    /// Returns the initial languages as a string.
    pub fn get_init_languages_as_string(&self) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetInitLanguagesAsString() returns a pointer to a C string
        let result = unsafe { TessBaseAPIGetInitLanguagesAsString(*handle) };
        if result.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            // SAFETY: We've verified result is non-null. CStr::from_ptr() is safe because:
            let c_str = unsafe { CStr::from_ptr(result) };
            Ok(c_str.to_str()?.to_owned())
        }
    }

    /// Gets the loaded languages as a vector of strings.
    ///
    /// # Returns
    ///
    /// Returns a vector of loaded languages.
    pub fn get_loaded_languages(&self) -> Result<Vec<String>> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        let vec_ptr = unsafe { TessBaseAPIGetLoadedLanguagesAsVector(*handle) };
        self.string_vec_to_rust(vec_ptr)
    }

    /// Gets the available languages as a vector of strings.
    ///
    /// # Returns
    ///
    /// Returns a vector of available languages.
    pub fn get_available_languages(&self) -> Result<Vec<String>> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        let vec_ptr = unsafe { TessBaseAPIGetAvailableLanguagesAsVector(*handle) };
        self.string_vec_to_rust(vec_ptr)
    }

    /// Converts a vector of C strings to a Rust vector of strings.
    ///
    /// # Arguments
    ///
    /// * `vec_ptr` - Pointer to the vector of C strings.
    ///
    /// # Returns
    ///
    /// Returns a vector of strings.
    fn string_vec_to_rust(&self, vec_ptr: *mut *mut c_char) -> Result<Vec<String>> {
        if vec_ptr.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        let mut result = Vec::new();
        let mut i = 0;
        loop {
            // SAFETY: We dereference vec_ptr at offset(i) to get a C string pointer.
            let str_ptr = unsafe { *vec_ptr.offset(i) };
            if str_ptr.is_null() {
                break;
            }
            // SAFETY: str_ptr is non-null (checked above) and points to a valid null-terminated
            let c_str = unsafe { CStr::from_ptr(str_ptr) };
            result.push(c_str.to_str()?.to_owned());
            i += 1;
        }
        // SAFETY: TessDeleteTextArray() deallocates both the array and all contained strings:
        unsafe { TessDeleteTextArray(vec_ptr) };
        Ok(result)
    }

    /// Clears the adaptive classifier.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if clearing the adaptive classifier is successful, otherwise returns an error.
    pub fn clear_adaptive_classifier(&self) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIClearAdaptiveClassifier() is safe because:
        unsafe { TessBaseAPIClearAdaptiveClassifier(*handle) };
        Ok(())
    }

    /// Clears the OCR engine.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if clearing the OCR engine is successful, otherwise returns an error.
    pub fn clear(&self) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIClear() is safe because:
        unsafe { TessBaseAPIClear(*handle) };
        Ok(())
    }

    /// Ends the OCR engine.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if ending the OCR engine is successful, otherwise returns an error.
    pub fn end(&self) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIEnd() is safe because:
        unsafe { TessBaseAPIEnd(*handle) };
        Ok(())
    }

    /// Checks if a word is valid.
    ///
    /// # Arguments
    ///
    /// * `word` - Word to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the word is valid, otherwise returns `false`.
    pub fn is_valid_word(&self, word: &str) -> Result<i32> {
        let word = CString::new(word).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIIsValidWord() is safe because:
        Ok(unsafe { TessBaseAPIIsValidWord(*handle, word.as_ptr()) })
    }

    /// Gets the text direction.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing the degrees and confidence.
    pub fn get_text_direction(&self) -> Result<(i32, f32)> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        let mut out_degrees = 0;
        let mut out_confidence = 0.0;
        // SAFETY: TessBaseAPIGetTextDirection() is safe because:
        unsafe {
            TessBaseAPIGetTextDirection(*handle, &mut out_degrees, &mut out_confidence);
        }
        Ok((out_degrees, out_confidence))
    }

    /// Initializes the OCR engine.
    ///
    /// # Arguments
    ///
    /// * `datapath` - Path to the data directory.
    /// * `language` - Language to use.
    /// * `oem` - OCR engine mode.
    /// * `configs` - Configuration strings.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initializing the OCR engine is successful, otherwise returns an error.
    pub fn init_1(&self, datapath: &str, language: &str, oem: i32, configs: &[&str]) -> Result<()> {
        let datapath = CString::new(datapath).map_err(|_| TesseractError::NullByteInString)?;
        let language = CString::new(language).map_err(|_| TesseractError::NullByteInString)?;
        let config_ptrs: Vec<_> = configs
            .iter()
            .map(|&s| CString::new(s).map_err(|_| TesseractError::NullByteInString))
            .collect::<Result<_>>()?;
        let config_ptr_ptrs: Vec<_> = config_ptrs.iter().map(|cs| cs.as_ptr()).collect();
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIInit1() is safe because:
        let result = unsafe {
            TessBaseAPIInit1(
                *handle,
                datapath.as_ptr(),
                language.as_ptr(),
                oem,
                config_ptr_ptrs.as_ptr(),
                config_ptrs.len() as c_int,
            )
        };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    /// Initializes the OCR engine.
    ///
    /// # Arguments
    ///
    /// * `datapath` - Path to the data directory.
    /// * `language` - Language to use.
    /// * `oem` - OCR engine mode.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initializing the OCR engine is successful, otherwise returns an error.
    pub fn init_2(&self, datapath: &str, language: &str, oem: i32) -> Result<()> {
        let datapath = CString::new(datapath).map_err(|_| TesseractError::NullByteInString)?;
        let language = CString::new(language).map_err(|_| TesseractError::NullByteInString)?;
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIInit2() is safe because:
        let result = unsafe { TessBaseAPIInit2(*handle, datapath.as_ptr(), language.as_ptr(), oem) };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    /// Initializes the OCR engine.
    ///
    /// # Arguments
    ///
    /// * `datapath` - Path to the data directory.
    /// * `language` - Language to use.
    /// * `oem` - OCR engine mode.
    /// * `configs` - Configuration strings.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initializing the OCR engine is successful, otherwise returns an error.
    pub fn init_4(&self, datapath: &str, language: &str, oem: i32, configs: &[&str]) -> Result<()> {
        let datapath = CString::new(datapath).map_err(|_| TesseractError::NullByteInString)?;
        let language = CString::new(language).map_err(|_| TesseractError::NullByteInString)?;
        let config_ptrs: Vec<_> = configs
            .iter()
            .map(|&s| CString::new(s).map_err(|_| TesseractError::NullByteInString))
            .collect::<Result<_>>()?;
        let config_ptr_ptrs: Vec<_> = config_ptrs.iter().map(|cs| cs.as_ptr()).collect();
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIInit4() is safe because:
        let result = unsafe {
            TessBaseAPIInit4(
                *handle,
                datapath.as_ptr(),
                language.as_ptr(),
                oem,
                config_ptr_ptrs.as_ptr(),
                config_ptrs.len() as c_int,
                std::ptr::null(),
                std::ptr::null(),
                0,
                0,
            )
        };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    /// Initializes the OCR engine.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw data.
    /// * `data_size` - Size of the data.
    /// * `language` - Language to use.
    /// * `oem` - OCR engine mode.
    /// * `configs` - Configuration strings.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initializing the OCR engine is successful, otherwise returns an error.
    pub fn init_5(&self, data: &[u8], data_size: i32, language: &str, oem: i32, configs: &[&str]) -> Result<()> {
        let language = CString::new(language).map_err(|_| TesseractError::NullByteInString)?;
        let config_ptrs: Vec<_> = configs
            .iter()
            .map(|&s| CString::new(s).map_err(|_| TesseractError::NullByteInString))
            .collect::<Result<_>>()?;
        let config_ptr_ptrs: Vec<_> = config_ptrs.iter().map(|cs| cs.as_ptr()).collect();
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIInit5() is safe because:
        let result = unsafe {
            TessBaseAPIInit5(
                *handle,
                data.as_ptr(),
                data_size,
                language.as_ptr(),
                oem,
                config_ptr_ptrs.as_ptr(),
                config_ptrs.len() as c_int,
                std::ptr::null(),
                std::ptr::null(),
                0,
                0,
            )
        };
        if result != 0 {
            Err(TesseractError::InitError)
        } else {
            Ok(())
        }
    }

    /// Sets the image for OCR processing.
    ///
    /// # Arguments
    ///
    /// * `image_data` - Raw image data.
    /// * `width` - Width of the image.
    /// * `height` - Height of the image.
    /// * `bytes_per_pixel` - Number of bytes per pixel (e.g., 3 for RGB, 1 for grayscale).
    /// * `bytes_per_line` - Number of bytes per line (usually width * bytes_per_pixel, but might be padded).
    pub fn set_image(
        &self,
        image_data: &[u8],
        width: i32,
        height: i32,
        bytes_per_pixel: i32,
        bytes_per_line: i32,
    ) -> Result<()> {
        if width <= 0 || height <= 0 {
            return Err(TesseractError::InvalidDimensions);
        }

        if bytes_per_pixel <= 0 {
            return Err(TesseractError::InvalidBytesPerPixel);
        }

        if bytes_per_line < width * bytes_per_pixel {
            return Err(TesseractError::InvalidBytesPerLine);
        }

        let expected_size = (height * bytes_per_line) as usize;
        if image_data.len() < expected_size {
            return Err(TesseractError::InvalidImageData);
        }

        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;

        // SAFETY: TessBaseAPISetImage() is safe because:
        unsafe {
            TessBaseAPISetImage(
                *handle,
                image_data.as_ptr(),
                width,
                height,
                bytes_per_pixel,
                bytes_per_line,
            );
        }
        Ok(())
    }

    /// Sets the image for OCR processing.
    ///
    /// # Arguments
    ///
    /// * `pix` - Pointer to the image data.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the image is successful, otherwise returns an error.
    pub fn set_image_2(&self, pix: *mut c_void) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetImage2() is safe because:
        unsafe { TessBaseAPISetImage2(*handle, pix) };
        Ok(())
    }

    /// Sets the source resolution for the image.
    ///
    /// # Arguments
    ///
    /// * `ppi` - PPI of the image.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the source resolution is successful, otherwise returns an error.
    pub fn set_source_resolution(&self, ppi: i32) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetSourceResolution() is safe because:
        unsafe { TessBaseAPISetSourceResolution(*handle, ppi) };
        Ok(())
    }

    /// Sets the rectangle for OCR processing.
    ///
    /// # Arguments
    ///
    /// * `left` - Left coordinate.
    /// * `top` - Top coordinate.
    /// * `width` - Width.
    /// * `height` - Height.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if setting the rectangle is successful, otherwise returns an error.
    pub fn set_rectangle(&self, left: i32, top: i32, width: i32, height: i32) -> Result<()> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPISetRectangle() is safe because:
        unsafe { TessBaseAPISetRectangle(*handle, left, top, width, height) };
        Ok(())
    }

    /// Performs OCR on the set image and returns the recognized text.
    ///
    /// # Returns
    ///
    /// Returns the recognized text as a String if successful, otherwise returns an error.
    pub fn get_utf8_text(&self) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;

        if *handle == std::ptr::null_mut() {
            return Err(TesseractError::UninitializedError);
        }

        // SAFETY: TessBaseAPIGetUTF8Text() returns a pointer to an allocated C string.
        let text_ptr = unsafe { TessBaseAPIGetUTF8Text(*handle) };
        if text_ptr.is_null() {
            return Err(TesseractError::OcrError);
        }

        // SAFETY: We've verified text_ptr is non-null. CStr::from_ptr() and TessDeleteText()
        let result = unsafe {
            let c_str = CStr::from_ptr(text_ptr);
            let result = c_str.to_str()?.to_owned();
            TessDeleteText(text_ptr);
            result
        };

        Ok(result)
    }

    /// Gets the iterator for the OCR results.
    ///
    /// # Returns
    ///
    /// Returns the iterator for the OCR results as a `ResultIterator` if successful, otherwise returns an error.
    pub fn get_iterator(&self) -> Result<ResultIterator> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetIterator() returns a pointer to a ResultIterator structure.
        let iterator = unsafe { TessBaseAPIGetIterator(*handle) };
        if iterator.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(ResultIterator::new(iterator))
        }
    }

    /// Gets the mutable iterator for the OCR results.
    ///
    /// # Returns
    ///
    /// Returns the mutable iterator for the OCR results as a `ResultIterator` if successful, otherwise returns an error.
    pub fn get_mutable_iterator(&self) -> Result<ResultIterator> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetMutableIterator() returns a pointer to a mutable ResultIterator.
        let iterator = unsafe { TessBaseAPIGetMutableIterator(*handle) };
        if iterator.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(ResultIterator::new(iterator))
        }
    }

    /// Analyzes the layout of the image.
    ///
    /// # Returns
    ///
    /// Returns the layout of the image as a `PageIterator` if successful, otherwise returns an error.
    pub fn analyse_layout(&self) -> Result<PageIterator> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIAnalyseLayout() returns a pointer to a PageIterator structure.
        let iterator = unsafe { TessBaseAPIAnalyseLayout(*handle) };
        if iterator.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            Ok(PageIterator::new(iterator))
        }
    }

    /// Gets the Unicode character for a given ID.
    ///
    /// # Arguments
    ///
    /// * `unichar_id` - ID of the Unicode character.
    ///
    /// # Returns
    ///
    /// Returns the Unicode character as a String if successful, otherwise returns an error.
    pub fn get_unichar(&self, unichar_id: i32) -> Result<String> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetUnichar() returns a pointer to a C string in Tesseract's memory.
        let char_ptr = unsafe { TessBaseAPIGetUnichar(*handle, unichar_id) };
        if char_ptr.is_null() {
            Err(TesseractError::NullPointerError)
        } else {
            // SAFETY: We've verified char_ptr is non-null. CStr::from_ptr() is safe because:
            let c_str = unsafe { CStr::from_ptr(char_ptr) };
            Ok(c_str.to_str()?.to_owned())
        }
    }

    /// Gets a page iterator for analyzing layout and getting bounding boxes
    pub fn analyze_layout(&self) -> Result<PageIterator> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIAnalyseLayout() returns a pointer to a PageIterator structure.
        let iterator = unsafe { TessBaseAPIAnalyseLayout(*handle) };
        if iterator.is_null() {
            return Err(TesseractError::NullPointerError);
        }
        Ok(PageIterator::new(iterator))
    }

    /// Get all word bounding boxes in a single FFI call.
    ///
    /// Calls `TessBaseAPIGetWords` and returns every word bounding box as a
    /// [`BoundingBoxArray`].  This is more efficient than iterating via
    /// [`get_iterator`](Self::get_iterator) when only bounding boxes are needed.
    ///
    /// # Returns
    ///
    /// Returns a [`BoundingBoxArray`] containing `(x, y, width, height)` for every
    /// word detected on the current page, or an error if the engine returns null.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kreuzberg_tesseract::TesseractAPI;
    /// # let api = TesseractAPI::new().unwrap();
    /// # api.init("/tessdata", "eng").unwrap();
    /// # api.set_image(&[], 1, 1, 1, 1).unwrap();
    /// let words = api.get_words().unwrap();
    /// println!("{} words found", words.len());
    /// ```
    pub fn get_words(&self) -> Result<BoundingBoxArray> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetWords() returns a newly-allocated BOXA* (Leptonica bounding-box
        let boxa = unsafe { TessBaseAPIGetWords(*handle, std::ptr::null_mut()) };
        if boxa.is_null() {
            return Ok(BoundingBoxArray {
                boxes: Vec::new(),
                block_ids: None,
                para_ids: None,
            });
        }
        // SAFETY: boxa is a valid non-null BOXA* returned by TessBaseAPIGetWords.
        let count = unsafe { boxaGetCount(boxa) };
        let mut boxes = Vec::with_capacity(count as usize);
        for i in 0..count {
            let mut x = 0_i32;
            let mut y = 0_i32;
            let mut w = 0_i32;
            let mut h = 0_i32;
            // SAFETY: boxaGetBox with L_NOCOPY (0) returns a borrowed pointer into the BOXA
            let bx = unsafe { boxaGetBox(boxa, i, 0) };
            if !bx.is_null() {
                let ok = unsafe { boxGetGeometry(bx, &mut x, &mut y, &mut w, &mut h) };
                if ok != 0 {
                    boxes.push((x, y, w, h));
                }
            }
        }
        // SAFETY: boxaDestroy takes a *mut *mut BOXA, sets the pointer to null, and frees the
        let mut boxa_mut = boxa;
        unsafe { boxaDestroy(&mut boxa_mut) };
        Ok(BoundingBoxArray {
            boxes,
            block_ids: None,
            para_ids: None,
        })
    }

    /// Get all region bounding boxes in a single FFI call.
    ///
    /// Calls `TessBaseAPIGetRegions` and returns every layout region as a
    /// [`BoundingBoxArray`].
    ///
    /// # Returns
    ///
    /// Returns a [`BoundingBoxArray`] containing `(x, y, width, height)` for every
    /// region on the current page, or an error if the engine returns null.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kreuzberg_tesseract::TesseractAPI;
    /// # let api = TesseractAPI::new().unwrap();
    /// # api.init("/tessdata", "eng").unwrap();
    /// # api.set_image(&[], 1, 1, 1, 1).unwrap();
    /// let regions = api.get_regions().unwrap();
    /// println!("{} regions found", regions.len());
    /// ```
    pub fn get_regions(&self) -> Result<BoundingBoxArray> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        // SAFETY: TessBaseAPIGetRegions() returns a newly-allocated BOXA* containing one BOX per
        let boxa = unsafe { TessBaseAPIGetRegions(*handle, std::ptr::null_mut()) };
        if boxa.is_null() {
            return Ok(BoundingBoxArray {
                boxes: Vec::new(),
                block_ids: None,
                para_ids: None,
            });
        }
        // SAFETY: See get_words() for full explanation — same pattern applies here.
        let count = unsafe { boxaGetCount(boxa) };
        let mut boxes = Vec::with_capacity(count as usize);
        for i in 0..count {
            let mut x = 0_i32;
            let mut y = 0_i32;
            let mut w = 0_i32;
            let mut h = 0_i32;
            let bx = unsafe { boxaGetBox(boxa, i, 0) };
            if !bx.is_null() {
                let ok = unsafe { boxGetGeometry(bx, &mut x, &mut y, &mut w, &mut h) };
                if ok != 0 {
                    boxes.push((x, y, w, h));
                }
            }
        }
        let mut boxa_mut = boxa;
        // SAFETY: boxaDestroy sets boxa_mut to null after freeing — called exactly once.
        unsafe { boxaDestroy(&mut boxa_mut) };
        Ok(BoundingBoxArray {
            boxes,
            block_ids: None,
            para_ids: None,
        })
    }

    /// Get all textline bounding boxes with block and paragraph IDs.
    ///
    /// Calls `TessBaseAPIGetTextlines1` with `raw_image=FALSE` and `raw_padding=0`,
    /// capturing both the `blockids` and `paraids` arrays alongside the bounding boxes.
    ///
    /// # Returns
    ///
    /// Returns a [`BoundingBoxArray`] where [`BoundingBoxArray::block_id`] and
    /// [`BoundingBoxArray::para_id`] return the corresponding IDs for each textline.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kreuzberg_tesseract::TesseractAPI;
    /// # let api = TesseractAPI::new().unwrap();
    /// # api.init("/tessdata", "eng").unwrap();
    /// # api.set_image(&[], 1, 1, 1, 1).unwrap();
    /// let lines = api.get_textlines().unwrap();
    /// for i in 0..lines.len() {
    ///     let (x, y, w, h) = lines.get(i).unwrap();
    ///     let block = lines.block_id(i).unwrap_or(-1);
    ///     println!("Line {i}: ({x},{y},{w},{h}) block={block}");
    /// }
    /// ```
    pub fn get_textlines(&self) -> Result<BoundingBoxArray> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        let mut blockids_ptr: *mut c_int = std::ptr::null_mut();
        let mut paraids_ptr: *mut c_int = std::ptr::null_mut();
        // SAFETY: TessBaseAPIGetTextlines1() returns a newly-allocated BOXA* and optionally
        let boxa = unsafe {
            TessBaseAPIGetTextlines1(*handle, 0, 0, std::ptr::null_mut(), &mut blockids_ptr, &mut paraids_ptr)
        };
        if boxa.is_null() {
            return Ok(BoundingBoxArray {
                boxes: Vec::new(),
                block_ids: None,
                para_ids: None,
            });
        }
        // SAFETY: See get_words() — same Leptonica traversal pattern.
        let count = unsafe { boxaGetCount(boxa) };
        let n = count as usize;
        let mut boxes = Vec::with_capacity(n);
        let mut block_ids_vec: Option<Vec<i32>> = if blockids_ptr.is_null() {
            None
        } else {
            Some(Vec::with_capacity(n))
        };
        let mut para_ids_vec: Option<Vec<i32>> = if paraids_ptr.is_null() {
            None
        } else {
            Some(Vec::with_capacity(n))
        };
        for i in 0..count {
            let mut x = 0_i32;
            let mut y = 0_i32;
            let mut w = 0_i32;
            let mut h = 0_i32;
            let bx = unsafe { boxaGetBox(boxa, i, 0) };
            if !bx.is_null() {
                let ok = unsafe { boxGetGeometry(bx, &mut x, &mut y, &mut w, &mut h) };
                if ok != 0 {
                    boxes.push((x, y, w, h));
                    // SAFETY: blockids_ptr/paraids_ptr are valid int arrays of `count`
                    if let Some(ref mut ids) = block_ids_vec {
                        ids.push(unsafe { *blockids_ptr.offset(i as isize) });
                    }
                    if let Some(ref mut ids) = para_ids_vec {
                        ids.push(unsafe { *paraids_ptr.offset(i as isize) });
                    }
                }
            }
        }
        // SAFETY: TessDeleteIntArray frees the Tesseract-allocated int array exactly once.
        if !blockids_ptr.is_null() {
            unsafe { TessDeleteIntArray(blockids_ptr) };
        }
        if !paraids_ptr.is_null() {
            unsafe { TessDeleteIntArray(paraids_ptr) };
        }
        let mut boxa_mut = boxa;
        // SAFETY: boxaDestroy sets boxa_mut to null after freeing — called exactly once.
        unsafe { boxaDestroy(&mut boxa_mut) };
        Ok(BoundingBoxArray {
            boxes,
            block_ids: block_ids_vec,
            para_ids: para_ids_vec,
        })
    }

    /// Get all component bounding boxes at the specified iterator level.
    ///
    /// Calls `TessBaseAPIGetComponentImages` and returns every matching component as a
    /// [`BoundingBoxArray`].
    ///
    /// # Arguments
    ///
    /// * `level` - The [`TessPageIteratorLevel`] granularity (block, paragraph, line, word, symbol).
    /// * `text_only` - If `true`, only text components are returned; if `false`, all components.
    ///
    /// # Returns
    ///
    /// Returns a [`BoundingBoxArray`] containing `(x, y, width, height)` for every
    /// matching component, or an error if the engine returns null.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use kreuzberg_tesseract::{TesseractAPI, TessPageIteratorLevel};
    /// # let api = TesseractAPI::new().unwrap();
    /// # api.init("/tessdata", "eng").unwrap();
    /// # api.set_image(&[], 1, 1, 1, 1).unwrap();
    /// let components = api.get_component_images(TessPageIteratorLevel::RIL_WORD, true).unwrap();
    /// println!("{} text components at word level", components.len());
    /// ```
    pub fn get_component_images(&self, level: TessPageIteratorLevel, text_only: bool) -> Result<BoundingBoxArray> {
        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;
        let mut blockids_ptr: *mut c_int = std::ptr::null_mut();
        // SAFETY: TessBaseAPIGetComponentImages() returns a newly-allocated BOXA* and optionally
        let boxa = unsafe {
            TessBaseAPIGetComponentImages(
                *handle,
                level as c_int,
                text_only as c_int,
                std::ptr::null_mut(),
                &mut blockids_ptr,
            )
        };
        if boxa.is_null() {
            return Ok(BoundingBoxArray {
                boxes: Vec::new(),
                block_ids: None,
                para_ids: None,
            });
        }
        // SAFETY: See get_words() — same Leptonica traversal pattern.
        let count = unsafe { boxaGetCount(boxa) };
        let n = count as usize;
        let mut boxes = Vec::with_capacity(n);
        let mut block_ids_vec: Option<Vec<i32>> = if blockids_ptr.is_null() {
            None
        } else {
            Some(Vec::with_capacity(n))
        };
        for i in 0..count {
            let mut x = 0_i32;
            let mut y = 0_i32;
            let mut w = 0_i32;
            let mut h = 0_i32;
            let bx = unsafe { boxaGetBox(boxa, i, 0) };
            if !bx.is_null() {
                let ok = unsafe { boxGetGeometry(bx, &mut x, &mut y, &mut w, &mut h) };
                if ok != 0 {
                    boxes.push((x, y, w, h));
                    // SAFETY: blockids_ptr is a valid int array of `count` elements; i < count.
                    if let Some(ref mut ids) = block_ids_vec {
                        ids.push(unsafe { *blockids_ptr.offset(i as isize) });
                    }
                }
            }
        }
        // SAFETY: TessDeleteIntArray frees the Tesseract-allocated int array exactly once.
        if !blockids_ptr.is_null() {
            unsafe { TessDeleteIntArray(blockids_ptr) };
        }
        let mut boxa_mut = boxa;
        // SAFETY: boxaDestroy sets boxa_mut to null after freeing — called exactly once.
        unsafe { boxaDestroy(&mut boxa_mut) };
        Ok(BoundingBoxArray {
            boxes,
            block_ids: block_ids_vec,
            para_ids: None,
        })
    }

    /// Gets both page and result iterators for full text analysis
    pub fn get_iterators(&self) -> Result<(PageIterator, ResultIterator)> {
        self.recognize()?;

        let handle = self.handle.lock().map_err(|_| TesseractError::MutexLockError)?;

        // SAFETY: TessBaseAPIAnalyseLayout() and TessBaseAPIGetIterator() both return pointers
        let page_iter = unsafe { TessBaseAPIAnalyseLayout(*handle) };
        let result_iter = unsafe { TessBaseAPIGetIterator(*handle) };

        if page_iter.is_null() || result_iter.is_null() {
            // SAFETY: If either iterator is null, we manually clean up the non-null one.
            if !page_iter.is_null() {
                unsafe { TessPageIteratorDelete(page_iter) };
            }
            if !result_iter.is_null() {
                unsafe { TessResultIteratorDelete(result_iter) };
            }
            return Err(TesseractError::NullPointerError);
        }

        Ok((PageIterator::new(page_iter), ResultIterator::new(result_iter)))
    }
}

#[cfg(any(feature = "build-tesseract", feature = "build-tesseract-wasm"))]
impl Drop for TesseractAPI {
    /// Drops the TesseractAPI instance.
    ///
    /// SAFETY: Drop must never panic, so we use `.ok()` to handle potential mutex poisoning.
    fn drop(&mut self) {
        if let Ok(handle) = self.handle.lock() {
            // SAFETY: We clean up the Tesseract handle by calling FFI functions in the correct order:
            unsafe {
                if !(*handle).is_null() {
                    TessBaseAPIEnd(*handle);
                    TessBaseAPIDelete(*handle);
                }
            }
        }
    }
}

#[cfg(any(feature = "build-tesseract", feature = "build-tesseract-wasm"))]
impl Clone for TesseractAPI {
    /// Clones the TesseractAPI instance, attempting to clone its configuration and state.
    ///
    /// If the mutex is poisoned, defaults to empty configuration.
    /// Initialization errors during cloning are silently ignored to prevent panics
    /// in Clone::clone() (which returns Self, not Result).
    fn clone(&self) -> Self {
        let config = self
            .config
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| TesseractConfiguration {
                datapath: String::new(),
                language: String::new(),
                variables: HashMap::new(),
            });

        let new_handle = unsafe { TessBaseAPICreate() };
        let new_api = TesseractAPI {
            handle: Arc::new(Mutex::new(new_handle)),
            config: Arc::new(Mutex::new(config.clone())),
        };

        if !config.datapath.is_empty() && new_api.init(&config.datapath, &config.language).is_ok() {
            for (name, value) in &config.variables {
                let _ = new_api.set_variable(name, value);
            }
        }

        new_api
    }
}

#[cfg(any(feature = "build-tesseract", feature = "build-tesseract-wasm"))]
ffi_extern! {
    fn TessBaseAPIMeanTextConf(handle: *mut c_void) -> c_int;
    fn TessBaseAPISetVariable(handle: *mut c_void, name: *const c_char, value: *const c_char) -> c_int;
    fn TessBaseAPIGetStringVariable(handle: *mut c_void, name: *const c_char) -> *const c_char;
    fn TessBaseAPIGetIntVariable(handle: *mut c_void, name: *const c_char) -> c_int;
    fn TessBaseAPIGetBoolVariable(handle: *mut c_void, name: *const c_char) -> c_int;
    fn TessBaseAPIGetDoubleVariable(handle: *mut c_void, name: *const c_char) -> c_double;
    fn TessBaseAPISetPageSegMode(handle: *mut c_void, mode: c_int);
    fn TessBaseAPIGetPageSegMode(handle: *mut c_void) -> c_int;
    fn TessBaseAPIRecognize(handle: *mut c_void, monitor: *mut c_void) -> c_int;
    fn TessBaseAPIGetHOCRText(handle: *mut c_void, page: c_int) -> *mut c_char;

    fn TessBaseAPIGetAltoText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetTsvText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetBoxText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetLSTMBoxText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetWordStrBoxText(handle: *mut c_void, page: c_int) -> *mut c_char;
    fn TessBaseAPIGetUNLVText(handle: *mut c_void) -> *mut c_char;
    fn TessBaseAPIAllWordConfidences(handle: *mut c_void) -> *const c_int;
    fn TessBaseAPIAdaptToWordStr(handle: *mut c_void, mode: c_int, wordstr: *const c_char) -> c_int;
    fn TessBaseAPIDetectOrientationScript(
        handle: *mut c_void,
        orient_deg: *mut c_int,
        orient_conf: *mut c_float,
        script_name: *mut *mut c_char,
        script_conf: *mut c_float,
    ) -> c_int;
    fn TessBaseAPISetMinOrientationMargin(handle: *mut c_void, margin: c_double);
    fn TessBaseAPIGetMutableIterator(handle: *mut c_void) -> *mut c_void;
    fn TessDeleteIntArray(arr: *const c_int);
    fn TessBaseAPISetInputImage(handle: *mut c_void, pix: *mut c_void);
    fn TessBaseAPIGetInputImage(handle: *mut c_void) -> *mut c_void;
    fn TessBaseAPISetOutputName(handle: *mut c_void, name: *const c_char);
    fn TessBaseAPISetDebugVariable(handle: *mut c_void, name: *const c_char, value: *const c_char) -> c_int;
    fn TessBaseAPIPrintVariablesToFile(handle: *mut c_void, filename: *const c_char) -> c_int;
    fn TessBaseAPIInitForAnalysePage(handle: *mut c_void);
    fn TessBaseAPIReadConfigFile(handle: *mut c_void, filename: *const c_char);
    fn TessBaseAPIReadDebugConfigFile(handle: *mut c_void, filename: *const c_char);
    fn TessBaseAPIGetThresholdedImageScaleFactor(handle: *mut c_void) -> c_int;
    fn TessBaseAPIAnalyseLayout(handle: *mut c_void) -> *mut c_void;
    fn TessBaseAPIGetInitLanguagesAsString(handle: *mut c_void) -> *const c_char;
    fn TessBaseAPIGetLoadedLanguagesAsVector(handle: *mut c_void) -> *mut *mut c_char;
    fn TessBaseAPIGetAvailableLanguagesAsVector(handle: *mut c_void) -> *mut *mut c_char;
    fn TessBaseAPIClearAdaptiveClassifier(handle: *mut c_void);
    fn TessDeleteTextArray(arr: *mut *mut c_char);

    fn TessVersion() -> *const c_char;
    fn TessBaseAPICreate() -> *mut c_void;
    fn TessBaseAPIDelete(handle: *mut c_void);
    fn TessBaseAPIInit3(handle: *mut c_void, datapath: *const c_char, language: *const c_char) -> c_int;
    fn TessBaseAPIInit1(
        handle: *mut c_void,
        datapath: *const c_char,
        language: *const c_char,
        oem: c_int,
        configs: *const *const c_char,
        configs_size: c_int,
    ) -> c_int;
    fn TessBaseAPIInit2(handle: *mut c_void, datapath: *const c_char, language: *const c_char, oem: c_int) -> c_int;
    fn TessBaseAPIInit4(
        handle: *mut c_void,
        datapath: *const c_char,
        language: *const c_char,
        oem: c_int,
        configs: *const *const c_char,
        configs_size: c_int,
        vars_vec: *const *const c_char,
        vars_values: *const *const c_char,
        vars_vec_size: usize,
        set_only_non_debug_params: c_int,
    ) -> c_int;
    fn TessBaseAPIInit5(
        handle: *mut c_void,
        data: *const u8,
        data_size: c_int,
        language: *const c_char,
        oem: c_int,
        configs: *const *const c_char,
        configs_size: c_int,
        vars_vec: *const *const c_char,
        vars_values: *const *const c_char,
        vars_vec_size: usize,
        set_only_non_debug_params: c_int,
    ) -> c_int;
    fn TessBaseAPISetImage(
        handle: *mut c_void,
        imagedata: *const u8,
        width: c_int,
        height: c_int,
        bytes_per_pixel: c_int,
        bytes_per_line: c_int,
    );
    fn TessBaseAPISetImage2(handle: *mut c_void, pix: *mut c_void);
    fn TessBaseAPISetSourceResolution(handle: *mut c_void, ppi: c_int);
    fn TessBaseAPISetRectangle(handle: *mut c_void, left: c_int, top: c_int, width: c_int, height: c_int);
    fn TessBaseAPIGetUTF8Text(handle: *mut c_void) -> *mut c_char;
    fn TessBaseAPIClear(handle: *mut c_void);
    fn TessBaseAPIEnd(handle: *mut c_void);
    fn TessBaseAPIIsValidWord(handle: *mut c_void, word: *const c_char) -> c_int;
    fn TessBaseAPIGetTextDirection(handle: *mut c_void, out_degrees: *mut c_int, out_confidence: *mut c_float);
    pub fn TessDeleteText(text: *mut c_char);

    fn TessBaseAPIGetUnichar(handle: *mut c_void, unichar_id: c_int) -> *const c_char;

    fn TessBaseAPIProcessPages(
        handle: *mut c_void,
        filename: *const c_char,
        retry_config: *const c_char,
        timeout_millisec: c_int,
        renderer: *mut c_void,
    ) -> *mut c_char;

    fn TessBaseAPIGetInputName(handle: *mut c_void) -> *const c_char;
    fn TessBaseAPISetInputName(handle: *mut c_void, name: *const c_char);
    fn TessBaseAPIGetSourceYResolution(handle: *mut c_void) -> c_int;
    fn TessBaseAPIGetDatapath(handle: *mut c_void) -> *const c_char;
    fn TessBaseAPIGetThresholdedImage(handle: *mut c_void) -> *mut c_void;

    fn TessBaseAPIGetWords(handle: *mut c_void, pixa: *mut *mut c_void) -> *mut c_void;
    fn TessBaseAPIGetRegions(handle: *mut c_void, pixa: *mut *mut c_void) -> *mut c_void;
    fn TessBaseAPIGetTextlines1(
        handle: *mut c_void,
        raw_image: c_int,
        raw_padding: c_int,
        pixa: *mut *mut c_void,
        blockids: *mut *mut c_int,
        paraids: *mut *mut c_int,
    ) -> *mut c_void;
    fn TessBaseAPIGetComponentImages(
        handle: *mut c_void,
        level: c_int,
        text_only: c_int,
        pixa: *mut *mut c_void,
        blockids: *mut *mut c_int,
    ) -> *mut c_void;

    fn boxaGetCount(boxa: *mut c_void) -> c_int;
    fn boxaGetBox(boxa: *mut c_void, index: c_int, accessflag: c_int) -> *mut c_void;
    fn boxGetGeometry(bx: *mut c_void, px: *mut c_int, py: *mut c_int, pw: *mut c_int, ph: *mut c_int) -> c_int;
    fn boxaDestroy(pboxa: *mut *mut c_void);

}
