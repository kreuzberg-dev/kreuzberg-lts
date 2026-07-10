package dev.kreuzberg;

/**
 * Exception thrown when an extraction operation is cancelled.
 *
 * <p>
 * This exception is raised when a user or system explicitly cancels an ongoing
 * extraction operation before it completes.
 *
 * @since 4.10.0
 */
public final class CancelledException extends KreuzbergException {
    private static final long serialVersionUID = 1L;

    /**
	 * Constructs a new CancelledException with the specified message.
	 *
	 * @param message
	 *            the detail message explaining why the operation was cancelled
	 */
    public CancelledException(String message) {
        super(message, ErrorCode.CANCELLED);
    }

    /**
	 * Constructs a new CancelledException with the specified message and cause.
	 *
	 * @param message
	 *            the detail message
	 * @param cause
	 *            the cause of the cancellation
	 */
    public CancelledException(String message, Throwable cause) {
        super(message, ErrorCode.CANCELLED, null, cause);
    }
}
