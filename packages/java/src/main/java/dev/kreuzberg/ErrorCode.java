package dev.kreuzberg;

/**
 * Enumeration of error codes returned by the Kreuzberg C library.
 *
 * <p>
 * Error codes indicate the type of error that occurred during FFI operations.
 * Use {@link KreuzbergException#getErrorCode()} to retrieve the code for a
 * specific exception.
 *
 * <p>
 * These codes match the panic_shield::ErrorCode enum in the Rust FFI layer
 * (codes 0-9). Higher-level error types like cache and image processing errors
 * are created by the Java binding through message-based classification and do
 * not have corresponding FFI error codes.
 *
 * @since 4.0.0
 */
public enum ErrorCode {
    /** No error occurred (0). */
    SUCCESS(0),

    /** Generic error that doesn't fit other categories (1). */
    GENERIC_ERROR(1),

    /** A panic occurred in the native library (2). */
    PANIC(2),

    /** Invalid argument provided to the library (3). */
    INVALID_ARGUMENT(3),

    /** I/O error during file or stream operations (4). */
    IO_ERROR(4),

    /** Error during document parsing (5). */
    PARSING_ERROR(5),

    /** Error during optical character recognition (6). */
    OCR_ERROR(6),

    /** Required dependency is missing (7). */
    MISSING_DEPENDENCY(7),

    /** Error during embedding generation (8). */
    EMBEDDING(8),

    /** Operation was cancelled (9). */
    CANCELLED(9);

    private final int code;

    ErrorCode(int code) {
        this.code = code;
    }

    /**
	 * Returns the numeric code value.
	 *
	 * @return the error code as an integer
	 */
    public int getCode() {
        return code;
    }

    private static final int CODE_GENERIC_ERROR = 1;
    private static final int CODE_PANIC = 2;
    private static final int CODE_INVALID_ARGUMENT = 3;
    private static final int CODE_IO_ERROR = 4;
    private static final int CODE_PARSING_ERROR = 5;
    private static final int CODE_OCR_ERROR = 6;
    private static final int CODE_MISSING_DEPENDENCY = 7;
    private static final int CODE_EMBEDDING = 8;
    private static final int CODE_CANCELLED = 9;

    /**
	 * Returns the ErrorCode for the given numeric code.
	 *
	 * @param code
	 *            the numeric error code
	 * @return the corresponding ErrorCode, or SUCCESS if code is not recognized
	 */
    public static ErrorCode fromCode(int code) {
        return switch (code) {
            case CODE_GENERIC_ERROR -> GENERIC_ERROR;
            case CODE_PANIC -> PANIC;
            case CODE_INVALID_ARGUMENT -> INVALID_ARGUMENT;
            case CODE_IO_ERROR -> IO_ERROR;
            case CODE_PARSING_ERROR -> PARSING_ERROR;
            case CODE_OCR_ERROR -> OCR_ERROR;
            case CODE_MISSING_DEPENDENCY -> MISSING_DEPENDENCY;
            case CODE_EMBEDDING -> EMBEDDING;
            case CODE_CANCELLED -> CANCELLED;
            default -> SUCCESS;
        };
    }

    /**
	 * Classifies an error message to determine the most appropriate error code.
	 *
	 * <p>
	 * This method analyzes the error message string to determine what type of error
	 * occurred. It's useful for mapping generic errors to more specific exception
	 * types based on message content.
	 *
	 * <p>
	 * Note: Cache and image processing errors are identified via message-based
	 * classification in {@link KreuzbergFFI} and mapped to specific exception types
	 * ({@link CacheException}, {@link ImageProcessingException}), but they return
	 * GENERIC_ERROR here since they don't have corresponding FFI error codes. This
	 * is consistent with how Python and Go handle these error types.
	 *
	 * @param message
	 *            the error message to classify
	 * @return the most appropriate ErrorCode based on message content
	 * @since 4.0.0
	 */
    public static ErrorCode classifyFromMessage(String message) {
        if (message == null || message.isBlank()) {
            return GENERIC_ERROR;
        }

        String lower = message.toLowerCase(java.util.Locale.ROOT);

        if (lower.contains("cancel") || lower.contains("cancelled")) {
            return CANCELLED;
        }

        if (lower.contains("parse") || lower.contains("parsing") || lower.contains("corrupt")
            || lower.contains("malformed")) {
            return PARSING_ERROR;
        }

        if (lower.contains("ocr") || lower.contains("tesseract") || lower.contains("recognition")) {
            return OCR_ERROR;
        }

        if (lower.contains("not found") || lower.contains("missing") || lower.contains("dependency")
            || lower.contains("not installed")) {
            return MISSING_DEPENDENCY;
        }

        if (lower.contains("io ") || lower.contains("i/o") || lower.contains("file") || lower.contains("permission")
            || lower.contains("access") || lower.contains("disk")) {
            return IO_ERROR;
        }

        if (lower.contains("invalid") || lower.contains("validation") || lower.contains("parameter")) {
            return INVALID_ARGUMENT;
        }

        if (lower.contains("embedding") || lower.contains("vector") || lower.contains("inference")
            || lower.contains("model")) {
            return EMBEDDING;
        }

        return GENERIC_ERROR;
    }
}
