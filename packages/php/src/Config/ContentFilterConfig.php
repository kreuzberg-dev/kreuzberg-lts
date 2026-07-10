<?php

declare(strict_types=1);

namespace Kreuzberg\Config;

/**
 * Content filtering configuration.
 *
 * Controls which content elements are included or excluded during extraction,
 * such as headers, footers, watermarks, and repeating text.
 *
 * @example
 * ```php
 * use Kreuzberg\Config\ContentFilterConfig;
 *
 * $filter = new ContentFilterConfig(
 *     includeHeaders: true,
 *     includeFooters: false,
 *     stripRepeatingText: true,
 *     includeWatermarks: false,
 * );
 * ```
 */
readonly class ContentFilterConfig
{
    /**
     * @param bool $includeHeaders Include page headers in extracted content. Default: false.
     * @param bool $includeFooters Include page footers in extracted content. Default: false.
     * @param bool $stripRepeatingText Strip repeating text (e.g., running headers/footers) from output. Default: true.
     * @param bool $includeWatermarks Include watermark text in extracted content. Default: false.
     */
    public function __construct(
        public bool $includeHeaders = false,
        public bool $includeFooters = false,
        public bool $stripRepeatingText = true,
        public bool $includeWatermarks = false,
    ) {}

    /**
     * Create configuration from array data.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var bool $includeHeaders */
        $includeHeaders = $data['include_headers'] ?? false;
        if (!is_bool($includeHeaders)) {
            /** @var bool $includeHeaders */
            $includeHeaders = (bool) $includeHeaders;
        }

        /** @var bool $includeFooters */
        $includeFooters = $data['include_footers'] ?? false;
        if (!is_bool($includeFooters)) {
            /** @var bool $includeFooters */
            $includeFooters = (bool) $includeFooters;
        }

        /** @var bool $stripRepeatingText */
        $stripRepeatingText = $data['strip_repeating_text'] ?? true;
        if (!is_bool($stripRepeatingText)) {
            /** @var bool $stripRepeatingText */
            $stripRepeatingText = (bool) $stripRepeatingText;
        }

        /** @var bool $includeWatermarks */
        $includeWatermarks = $data['include_watermarks'] ?? false;
        if (!is_bool($includeWatermarks)) {
            /** @var bool $includeWatermarks */
            $includeWatermarks = (bool) $includeWatermarks;
        }

        return new self(
            includeHeaders: $includeHeaders,
            includeFooters: $includeFooters,
            stripRepeatingText: $stripRepeatingText,
            includeWatermarks: $includeWatermarks,
        );
    }

    /**
     * Convert configuration to array for FFI.
     *
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [];

        if ($this->includeHeaders) {
            $result['include_headers'] = true;
        }
        if ($this->includeFooters) {
            $result['include_footers'] = true;
        }
        if (!$this->stripRepeatingText) {
            $result['strip_repeating_text'] = false;
        }
        if ($this->includeWatermarks) {
            $result['include_watermarks'] = true;
        }

        return $result;
    }
}
