package dev.kreuzberg.config;

import java.util.HashMap;
import java.util.Map;

/**
 * Content filtering configuration.
 *
 * <p>
 * Controls which content elements are included or excluded during extraction,
 * such as headers, footers, watermarks, and repeating text.
 *
 * @since 4.8.0
 */
public final class ContentFilterConfig {
    private final Boolean includeHeaders;
    private final Boolean includeFooters;
    private final Boolean stripRepeatingText;
    private final Boolean includeWatermarks;

    private ContentFilterConfig(Builder builder) {
        this.includeHeaders = builder.includeHeaders;
        this.includeFooters = builder.includeFooters;
        this.stripRepeatingText = builder.stripRepeatingText;
        this.includeWatermarks = builder.includeWatermarks;
    }

    public static Builder builder() {
        return new Builder();
    }

    /**
	 * Get whether page headers are included in extracted content.
	 *
	 * @return true if headers are included, or null if not set (defaults to false)
	 */
    public Boolean getIncludeHeaders() {
        return includeHeaders;
    }

    /**
	 * Get whether page footers are included in extracted content.
	 *
	 * @return true if footers are included, or null if not set (defaults to false)
	 */
    public Boolean getIncludeFooters() {
        return includeFooters;
    }

    /**
	 * Get whether repeating text is stripped from output.
	 *
	 * @return true if repeating text is stripped, or null if not set (defaults to true)
	 */
    public Boolean getStripRepeatingText() {
        return stripRepeatingText;
    }

    /**
	 * Get whether watermark text is included in extracted content.
	 *
	 * @return true if watermarks are included, or null if not set (defaults to false)
	 */
    public Boolean getIncludeWatermarks() {
        return includeWatermarks;
    }

    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        if (includeHeaders != null) {
            map.put("include_headers", includeHeaders);
        }
        if (includeFooters != null) {
            map.put("include_footers", includeFooters);
        }
        if (stripRepeatingText != null) {
            map.put("strip_repeating_text", stripRepeatingText);
        }
        if (includeWatermarks != null) {
            map.put("include_watermarks", includeWatermarks);
        }
        return map;
    }

    public static final class Builder {
        private Boolean includeHeaders;
        private Boolean includeFooters;
        private Boolean stripRepeatingText;
        private Boolean includeWatermarks;

        private Builder() {
        }

        /**
		 * Set whether to include page headers in extracted content.
		 *
		 * @param includeHeaders
		 *            true to include headers
		 * @return this builder for chaining
		 */
        public Builder includeHeaders(Boolean includeHeaders) {
            this.includeHeaders = includeHeaders;
            return this;
        }

        /**
		 * Set whether to include page footers in extracted content.
		 *
		 * @param includeFooters
		 *            true to include footers
		 * @return this builder for chaining
		 */
        public Builder includeFooters(Boolean includeFooters) {
            this.includeFooters = includeFooters;
            return this;
        }

        /**
		 * Set whether to strip repeating text from output.
		 *
		 * @param stripRepeatingText
		 *            true to strip repeating text
		 * @return this builder for chaining
		 */
        public Builder stripRepeatingText(Boolean stripRepeatingText) {
            this.stripRepeatingText = stripRepeatingText;
            return this;
        }

        /**
		 * Set whether to include watermark text in extracted content.
		 *
		 * @param includeWatermarks
		 *            true to include watermarks
		 * @return this builder for chaining
		 */
        public Builder includeWatermarks(Boolean includeWatermarks) {
            this.includeWatermarks = includeWatermarks;
            return this;
        }

        public ContentFilterConfig build() {
            return new ContentFilterConfig(this);
        }
    }

    static ContentFilterConfig fromMap(Map<String, Object> map) {
        if (map == null) {
            return null;
        }
        Builder builder = builder();
        Object includeHeadersValue = map.get("include_headers");
        if (includeHeadersValue instanceof Boolean) {
            builder.includeHeaders((Boolean) includeHeadersValue);
        }
        Object includeFootersValue = map.get("include_footers");
        if (includeFootersValue instanceof Boolean) {
            builder.includeFooters((Boolean) includeFootersValue);
        }
        Object stripRepeatingTextValue = map.get("strip_repeating_text");
        if (stripRepeatingTextValue instanceof Boolean) {
            builder.stripRepeatingText((Boolean) stripRepeatingTextValue);
        }
        Object includeWatermarksValue = map.get("include_watermarks");
        if (includeWatermarksValue instanceof Boolean) {
            builder.includeWatermarks((Boolean) includeWatermarksValue);
        }
        return builder.build();
    }
}
