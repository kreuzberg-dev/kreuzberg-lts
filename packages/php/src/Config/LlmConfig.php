<?php

declare(strict_types=1);

namespace Kreuzberg\Config;

/**
 * LLM provider/model configuration for liter-llm integration.
 *
 * Each feature (VLM OCR, structured extraction) carries its own LlmConfig,
 * allowing different providers per feature.
 *
 * @example
 * ```php
 * use Kreuzberg\Config\LlmConfig;
 *
 * $llm = new LlmConfig(
 *     model: 'openai/gpt-4o',
 *     temperature: 0.0,
 * );
 * ```
 */
readonly class LlmConfig
{
    /**
     * @param string $model Provider/model string using liter-llm routing format
     *                      (e.g., "openai/gpt-4o", "anthropic/claude-sonnet-4-20250514")
     * @param string|null $apiKey API key for the provider (falls back to env var if null)
     * @param string|null $baseUrl Custom base URL override for the provider endpoint
     * @param int|null $timeoutSecs Request timeout in seconds (default: 60)
     * @param int|null $maxRetries Maximum retry attempts (default: 3)
     * @param float|null $temperature Sampling temperature for generation tasks
     * @param int|null $maxTokens Maximum tokens to generate
     */
    public function __construct(
        public string $model,
        public ?string $apiKey = null,
        public ?string $baseUrl = null,
        public ?int $timeoutSecs = null,
        public ?int $maxRetries = null,
        public ?float $temperature = null,
        public ?int $maxTokens = null,
    ) {}

    /**
     * Create configuration from array data.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var string $model */
        $model = $data['model'] ?? '';
        if (!is_string($model)) {
            /** @var string $model */
            $model = (string) $model;
        }

        /** @var string|null $apiKey */
        $apiKey = $data['api_key'] ?? null;
        if ($apiKey !== null && !is_string($apiKey)) {
            /** @var string $apiKey */
            $apiKey = (string) $apiKey;
        }

        /** @var string|null $baseUrl */
        $baseUrl = $data['base_url'] ?? null;
        if ($baseUrl !== null && !is_string($baseUrl)) {
            /** @var string $baseUrl */
            $baseUrl = (string) $baseUrl;
        }

        /** @var int|null $timeoutSecs */
        $timeoutSecs = $data['timeout_secs'] ?? null;
        if ($timeoutSecs !== null && !is_int($timeoutSecs)) {
            /** @var int $timeoutSecs */
            $timeoutSecs = (int) $timeoutSecs;
        }

        /** @var int|null $maxRetries */
        $maxRetries = $data['max_retries'] ?? null;
        if ($maxRetries !== null && !is_int($maxRetries)) {
            /** @var int $maxRetries */
            $maxRetries = (int) $maxRetries;
        }

        /** @var float|null $temperature */
        $temperature = $data['temperature'] ?? null;
        if ($temperature !== null && !is_float($temperature)) {
            /** @var float $temperature */
            $temperature = (float) $temperature;
        }

        /** @var int|null $maxTokens */
        $maxTokens = $data['max_tokens'] ?? null;
        if ($maxTokens !== null && !is_int($maxTokens)) {
            /** @var int $maxTokens */
            $maxTokens = (int) $maxTokens;
        }

        return new self(
            model: $model,
            apiKey: $apiKey,
            baseUrl: $baseUrl,
            timeoutSecs: $timeoutSecs,
            maxRetries: $maxRetries,
            temperature: $temperature,
            maxTokens: $maxTokens,
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
            'model' => $this->model,
        ];

        if ($this->apiKey !== null) {
            $result['api_key'] = $this->apiKey;
        }
        if ($this->baseUrl !== null) {
            $result['base_url'] = $this->baseUrl;
        }
        if ($this->timeoutSecs !== null) {
            $result['timeout_secs'] = $this->timeoutSecs;
        }
        if ($this->maxRetries !== null) {
            $result['max_retries'] = $this->maxRetries;
        }
        if ($this->temperature !== null) {
            $result['temperature'] = $this->temperature;
        }
        if ($this->maxTokens !== null) {
            $result['max_tokens'] = $this->maxTokens;
        }

        return $result;
    }
}
