<?php

declare(strict_types=1);

namespace Kreuzberg\Config;

/**
 * Configuration for LLM-based structured data extraction.
 *
 * Sends extracted document content to a VLM with a JSON schema,
 * returning structured data that conforms to the schema.
 *
 * @example
 * ```php
 * use Kreuzberg\Config\StructuredExtractionConfig;
 * use Kreuzberg\Config\LlmConfig;
 *
 * $config = new StructuredExtractionConfig(
 *     schema: [
 *         'type' => 'object',
 *         'properties' => [
 *             'vendor' => ['type' => 'string'],
 *             'total' => ['type' => 'number'],
 *         ],
 *         'required' => ['vendor', 'total'],
 *     ],
 *     llm: new LlmConfig(model: 'openai/gpt-4o'),
 *     schemaName: 'invoice_data',
 *     strict: true,
 * );
 * ```
 */
readonly class StructuredExtractionConfig
{
    /**
     * @param array<string, mixed> $schema JSON Schema defining the desired output structure
     * @param LlmConfig $llm LLM configuration for the extraction
     * @param string $schemaName Schema name passed to the LLM's structured output mode (default: "extraction")
     * @param string|null $schemaDescription Optional schema description for the LLM
     * @param bool $strict Enable strict mode -- output must exactly match the schema (default: false)
     * @param string|null $prompt Custom extraction prompt template (Jinja2 format)
     */
    public function __construct(
        public array $schema,
        public LlmConfig $llm,
        public string $schemaName = 'extraction',
        public ?string $schemaDescription = null,
        public bool $strict = false,
        public ?string $prompt = null,
    ) {}

    /**
     * Create configuration from array data.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var array<string, mixed> $schema */
        $schema = $data['schema'] ?? [];
        if (!is_array($schema)) {
            /** @var array<string, mixed> $schema */
            $schema = (array) $schema;
        }

        /** @var array<string, mixed> $llmData */
        $llmData = $data['llm'] ?? [];
        if (!is_array($llmData)) {
            /** @var array<string, mixed> $llmData */
            $llmData = (array) $llmData;
        }
        $llm = LlmConfig::fromArray($llmData);

        /** @var string $schemaName */
        $schemaName = $data['schema_name'] ?? 'extraction';
        if (!is_string($schemaName)) {
            /** @var string $schemaName */
            $schemaName = (string) $schemaName;
        }

        /** @var string|null $schemaDescription */
        $schemaDescription = $data['schema_description'] ?? null;
        if ($schemaDescription !== null && !is_string($schemaDescription)) {
            /** @var string $schemaDescription */
            $schemaDescription = (string) $schemaDescription;
        }

        /** @var bool $strict */
        $strict = $data['strict'] ?? false;
        if (!is_bool($strict)) {
            /** @var bool $strict */
            $strict = (bool) $strict;
        }

        /** @var string|null $prompt */
        $prompt = $data['prompt'] ?? null;
        if ($prompt !== null && !is_string($prompt)) {
            /** @var string $prompt */
            $prompt = (string) $prompt;
        }

        return new self(
            schema: $schema,
            llm: $llm,
            schemaName: $schemaName,
            schemaDescription: $schemaDescription,
            strict: $strict,
            prompt: $prompt,
        );
    }

    /**
     * Convert configuration to array for FFI.
     *
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        $result = [
            'schema' => $this->schema,
            'llm' => $this->llm->toArray(),
            'schema_name' => $this->schemaName,
        ];

        if ($this->schemaDescription !== null) {
            $result['schema_description'] = $this->schemaDescription;
        }
        if ($this->strict) {
            $result['strict'] = true;
        }
        if ($this->prompt !== null) {
            $result['prompt'] = $this->prompt;
        }

        return $result;
    }
}
