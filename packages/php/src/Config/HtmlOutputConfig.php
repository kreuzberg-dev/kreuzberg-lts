<?php

declare(strict_types=1);

namespace Kreuzberg\Config;

/**
 * Configuration for styled HTML output rendering.
 *
 * Controls how extraction results are rendered as styled HTML,
 * including theme selection, CSS customization, and class prefix configuration.
 *
 * @example
 * ```php
 * use Kreuzberg\Config\HtmlOutputConfig;
 *
 * $config = new HtmlOutputConfig(
 *     theme: 'github',
 *     classPrefix: 'kb-',
 *     embedCss: true,
 *     css: '.kb-p { color: red; }',
 * );
 * ```
 */
readonly class HtmlOutputConfig
{
    /**
     * @param string|null $css Custom CSS string to apply. When set, appended after theme CSS.
     * @param string|null $cssFile Path to a CSS file to load. When set, file contents are appended after theme CSS.
     * @param string $theme HTML theme preset. One of: "default", "github", "dark", "light", "unstyled".
     * @param string $classPrefix CSS class prefix for generated HTML elements. Default: "kb-".
     * @param bool $embedCss Whether to embed CSS inline in the HTML output. Default: true.
     */
    public function __construct(
        public ?string $css = null,
        public ?string $cssFile = null,
        public string $theme = 'default',
        public string $classPrefix = 'kb-',
        public bool $embedCss = true,
    ) {}

    /**
     * Create configuration from array data.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var string|null $css */
        $css = $data['css'] ?? null;
        if ($css !== null && !is_string($css)) {
            /** @var string $css */
            $css = (string) $css;
        }

        /** @var string|null $cssFile */
        $cssFile = $data['css_file'] ?? null;
        if ($cssFile !== null && !is_string($cssFile)) {
            /** @var string $cssFile */
            $cssFile = (string) $cssFile;
        }

        /** @var string $theme */
        $theme = $data['theme'] ?? 'default';
        if (!is_string($theme)) {
            /** @var string $theme */
            $theme = (string) $theme;
        }

        /** @var string $classPrefix */
        $classPrefix = $data['class_prefix'] ?? 'kb-';
        if (!is_string($classPrefix)) {
            /** @var string $classPrefix */
            $classPrefix = (string) $classPrefix;
        }

        /** @var bool $embedCss */
        $embedCss = $data['embed_css'] ?? true;
        if (!is_bool($embedCss)) {
            /** @var bool $embedCss */
            $embedCss = (bool) $embedCss;
        }

        return new self(css: $css, cssFile: $cssFile, theme: $theme, classPrefix: $classPrefix, embedCss: $embedCss);
    }

    /**
     * Convert configuration to array for FFI.
     *
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [];

        if ($this->css !== null) {
            $result['css'] = $this->css;
        }
        if ($this->cssFile !== null) {
            $result['css_file'] = $this->cssFile;
        }
        if ($this->theme !== 'default') {
            $result['theme'] = $this->theme;
        }
        if ($this->classPrefix !== 'kb-') {
            $result['class_prefix'] = $this->classPrefix;
        }
        if (!$this->embedCss) {
            $result['embed_css'] = false;
        }

        return $result;
    }
}
