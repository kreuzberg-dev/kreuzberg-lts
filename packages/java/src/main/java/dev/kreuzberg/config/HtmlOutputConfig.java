package dev.kreuzberg.config;

import java.util.HashMap;
import java.util.Map;

/**
 * HTML output configuration for styled HTML rendering.
 *
 * <p>
 * Controls how {@code OutputFormat.HTML} renders extraction results with
 * CSS themes, custom stylesheets, and class prefixes.
 *
 * @since 4.8.1
 */
public final class HtmlOutputConfig {
    private final String css;
    private final String cssFile;
    private final String theme;
    private final String classPrefix;
    private final Boolean embedCss;

    private HtmlOutputConfig(Builder builder) {
        this.css = builder.css;
        this.cssFile = builder.cssFile;
        this.theme = builder.theme;
        this.classPrefix = builder.classPrefix;
        this.embedCss = builder.embedCss;
    }

    /**
	 * Creates a new builder for HTML output configuration.
	 *
	 * @return a new builder instance
	 */
    public static Builder builder() {
        return new Builder();
    }

    /**
	 * Gets the inline CSS string.
	 *
	 * @return the inline CSS, or null if not set
	 */
    public String getCss() {
        return css;
    }

    /**
	 * Gets the path to a CSS file.
	 *
	 * @return the CSS file path, or null if not set
	 */
    public String getCssFile() {
        return cssFile;
    }

    /**
	 * Gets the HTML theme name.
	 *
	 * @return the theme (default, github, dark, light, unstyled), or null if not set
	 */
    public String getTheme() {
        return theme;
    }

    /**
	 * Gets the CSS class prefix.
	 *
	 * @return the class prefix, or null if not set (defaults to "kb-")
	 */
    public String getClassPrefix() {
        return classPrefix;
    }

    /**
	 * Gets whether CSS is embedded in the output.
	 *
	 * @return true if CSS is embedded, or null if not set (defaults to true)
	 */
    public Boolean getEmbedCss() {
        return embedCss;
    }

    /**
	 * Converts this configuration to a map for FFI.
	 *
	 * @return a map representation
	 */
    public Map<String, Object> toMap() {
        Map<String, Object> map = new HashMap<>();
        if (css != null) {
            map.put("css", css);
        }
        if (cssFile != null) {
            map.put("css_file", cssFile);
        }
        if (theme != null) {
            map.put("theme", theme);
        }
        if (classPrefix != null) {
            map.put("class_prefix", classPrefix);
        }
        if (embedCss != null) {
            map.put("embed_css", embedCss);
        }
        return map;
    }

    static HtmlOutputConfig fromMap(Map<String, Object> map) {
        if (map == null) {
            return null;
        }
        Builder builder = builder();
        Object cssValue = map.get("css");
        if (cssValue instanceof String) {
            builder.css((String) cssValue);
        }
        Object cssFileValue = map.get("css_file");
        if (cssFileValue instanceof String) {
            builder.cssFile((String) cssFileValue);
        }
        Object themeValue = map.get("theme");
        if (themeValue instanceof String) {
            builder.theme((String) themeValue);
        }
        Object classPrefixValue = map.get("class_prefix");
        if (classPrefixValue instanceof String) {
            builder.classPrefix((String) classPrefixValue);
        }
        Object embedCssValue = map.get("embed_css");
        if (embedCssValue instanceof Boolean) {
            builder.embedCss((Boolean) embedCssValue);
        }
        return builder.build();
    }

    /** Builder for {@link HtmlOutputConfig}. */
    public static final class Builder {
        private String css;
        private String cssFile;
        private String theme;
        private String classPrefix;
        private Boolean embedCss;

        private Builder() {
        }

        /**
		 * Sets the inline CSS string.
		 *
		 * @param css
		 *            the inline CSS
		 * @return this builder
		 */
        public Builder css(String css) {
            this.css = css;
            return this;
        }

        /**
		 * Sets the path to a CSS file.
		 *
		 * @param cssFile
		 *            the CSS file path
		 * @return this builder
		 */
        public Builder cssFile(String cssFile) {
            this.cssFile = cssFile;
            return this;
        }

        /**
		 * Sets the HTML theme.
		 *
		 * @param theme
		 *            the theme name (default, github, dark, light, unstyled)
		 * @return this builder
		 */
        public Builder theme(String theme) {
            this.theme = theme;
            return this;
        }

        /**
		 * Sets the CSS class prefix.
		 *
		 * @param classPrefix
		 *            the class prefix (default: "kb-")
		 * @return this builder
		 */
        public Builder classPrefix(String classPrefix) {
            this.classPrefix = classPrefix;
            return this;
        }

        /**
		 * Sets whether to embed CSS in the output.
		 *
		 * @param embedCss
		 *            true to embed CSS (default: true)
		 * @return this builder
		 */
        public Builder embedCss(Boolean embedCss) {
            this.embedCss = embedCss;
            return this;
        }

        /**
		 * Builds the HTML output configuration.
		 *
		 * @return the built configuration
		 */
        public HtmlOutputConfig build() {
            return new HtmlOutputConfig(this);
        }
    }
}
