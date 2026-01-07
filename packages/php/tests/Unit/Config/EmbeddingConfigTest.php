<?php

declare(strict_types=1);

namespace Kreuzberg\Tests\Unit\Config;

use Kreuzberg\Config\EmbeddingConfig;
use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\Attributes\Group;
use PHPUnit\Framework\Attributes\Test;
use PHPUnit\Framework\TestCase;

/**
 * Unit tests for EmbeddingConfig readonly class.
 *
 * Tests construction, serialization, factory methods, readonly enforcement,
 * and handling of string, boolean, and nullable integer properties.
 *
 * Test Coverage:
 * - Construction with default values
 * - Construction with custom values
 * - toArray() serialization with optional field inclusion
 * - fromArray() factory method
 * - fromJson() factory method
 * - toJson() serialization
 * - Readonly enforcement
 * - Null handling
 * - Invalid JSON handling
 * - Round-trip serialization
 */
#[CoversClass(EmbeddingConfig::class)]
#[Group('unit')]
#[Group('config')]
final class EmbeddingConfigTest extends TestCase
{
    #[Test]
    public function it_creates_with_default_values(): void
    {
        $config = new EmbeddingConfig();

        $this->assertSame('balanced', $config->model);
        $this->assertTrue($config->normalize);
        $this->assertNull($config->batchSize);
    }

    #[Test]
    public function it_creates_with_custom_values(): void
    {
        $config = new EmbeddingConfig(
            model: 'sentence-transformers/all-mpnet-base-v2',
            normalize: false,
            batchSize: 32,
        );

        $this->assertSame('sentence-transformers/all-mpnet-base-v2', $config->model);
        $this->assertFalse($config->normalize);
        $this->assertSame(32, $config->batchSize);
    }

    #[Test]
    public function it_serializes_to_array_with_only_non_default_values(): void
    {
        $config = new EmbeddingConfig(model: 'custom-model', normalize: true);
        $array = $config->toArray();

        $this->assertIsArray($array);
        $this->assertArrayHasKey('model', $array);
        $this->assertSame('custom-model', $array['model']);
        $this->assertTrue($array['normalize']);
        $this->assertArrayNotHasKey('batch_size', $array);
    }

    #[Test]
    public function it_includes_batch_size_in_array_when_set(): void
    {
        $config = new EmbeddingConfig(
            model: 'test-model',
            batchSize: 64,
        );
        $array = $config->toArray();

        $this->assertArrayHasKey('batch_size', $array);
        $this->assertSame(64, $array['batch_size']);
    }

    #[Test]
    public function it_creates_from_array_with_defaults(): void
    {
        $config = EmbeddingConfig::fromArray([]);

        $this->assertSame('balanced', $config->model);
        $this->assertTrue($config->normalize);
        $this->assertNull($config->batchSize);
    }

    #[Test]
    public function it_creates_from_array_with_all_fields(): void
    {
        $data = [
            'model' => 'bert-base-uncased',
            'normalize' => false,
            'batch_size' => 128,
        ];
        $config = EmbeddingConfig::fromArray($data);

        $this->assertSame('bert-base-uncased', $config->model);
        $this->assertFalse($config->normalize);
        $this->assertSame(128, $config->batchSize);
    }

    #[Test]
    public function it_serializes_to_json(): void
    {
        $config = new EmbeddingConfig(
            model: 'distilbert-base-uncased',
            normalize: true,
            batchSize: 16,
        );
        $json = $config->toJson();

        $this->assertJson($json);
        $decoded = json_decode($json, true);

        $this->assertSame('distilbert-base-uncased', $decoded['model']);
        $this->assertTrue($decoded['normalize']);
        $this->assertSame(16, $decoded['batch_size']);
    }

    #[Test]
    public function it_creates_from_json(): void
    {
        $json = json_encode([
            'model' => 'roberta-base',
            'normalize' => false,
            'batch_size' => 256,
        ]);
        $config = EmbeddingConfig::fromJson($json);

        $this->assertSame('roberta-base', $config->model);
        $this->assertFalse($config->normalize);
        $this->assertSame(256, $config->batchSize);
    }

    #[Test]
    public function it_round_trips_through_json(): void
    {
        $original = new EmbeddingConfig(
            model: 'xlm-roberta-base',
            normalize: true,
            batchSize: 512,
        );

        $json = $original->toJson();
        $restored = EmbeddingConfig::fromJson($json);

        $this->assertSame($original->model, $restored->model);
        $this->assertSame($original->normalize, $restored->normalize);
        $this->assertSame($original->batchSize, $restored->batchSize);
    }

    #[Test]
    public function it_throws_on_invalid_json(): void
    {
        $this->expectException(\InvalidArgumentException::class);
        $this->expectExceptionMessage('Invalid JSON');

        EmbeddingConfig::fromJson('{ invalid }');
    }

    #[Test]
    public function it_enforces_readonly_on_model_property(): void
    {
        $this->expectException(\Error::class);

        $config = new EmbeddingConfig(model: 'model-1');
        $config->model = 'model-2';
    }

    #[Test]
    public function it_enforces_readonly_on_normalize_property(): void
    {
        $this->expectException(\Error::class);

        $config = new EmbeddingConfig(normalize: true);
        $config->normalize = false;
    }

    #[Test]
    public function it_enforces_readonly_on_batch_size_property(): void
    {
        $this->expectException(\Error::class);

        $config = new EmbeddingConfig(batchSize: 32);
        $config->batchSize = 64;
    }

    #[Test]
    public function it_creates_from_file(): void
    {
        $tempFile = tempnam(sys_get_temp_dir(), 'emb_');
        if ($tempFile === false) {
            $this->markTestSkipped('Unable to create temporary file');
        }

        try {
            file_put_contents($tempFile, json_encode([
                'model' => 'test-model',
                'normalize' => false,
                'batch_size' => 64,
            ]));

            $config = EmbeddingConfig::fromFile($tempFile);

            $this->assertSame('test-model', $config->model);
            $this->assertFalse($config->normalize);
            $this->assertSame(64, $config->batchSize);
        } finally {
            if (file_exists($tempFile)) {
                unlink($tempFile);
            }
        }
    }

    #[Test]
    public function it_throws_when_file_not_found(): void
    {
        $this->expectException(\InvalidArgumentException::class);
        $this->expectExceptionMessage('File not found');

        EmbeddingConfig::fromFile('/nonexistent/path/config.json');
    }

    #[Test]
    public function it_handles_type_coercion_for_model(): void
    {
        $data = ['model' => 123];
        $config = EmbeddingConfig::fromArray($data);

        $this->assertIsString($config->model);
        $this->assertSame('123', $config->model);
    }

    #[Test]
    public function it_handles_type_coercion_for_normalize(): void
    {
        $data = ['normalize' => 0];
        $config = EmbeddingConfig::fromArray($data);

        $this->assertIsBool($config->normalize);
        $this->assertFalse($config->normalize);
    }

    #[Test]
    public function it_handles_type_coercion_for_batch_size(): void
    {
        $data = ['batch_size' => '256'];
        $config = EmbeddingConfig::fromArray($data);

        $this->assertIsInt($config->batchSize);
        $this->assertSame(256, $config->batchSize);
    }

    #[Test]
    public function it_supports_various_model_names(): void
    {
        $models = [
            'fast',
            'balanced',
            'quality',
            'multilingual',
        ];

        foreach ($models as $model) {
            $config = new EmbeddingConfig(model: $model);
            $this->assertSame($model, $config->model);

            $array = $config->toArray();
            $this->assertSame($model, $array['model']);
        }
    }

    #[Test]
    public function it_supports_large_batch_sizes(): void
    {
        $config = new EmbeddingConfig(batchSize: 10000);

        $this->assertSame(10000, $config->batchSize);
        $array = $config->toArray();
        $this->assertArrayHasKey('batch_size', $array);
        $this->assertSame(10000, $array['batch_size']);
    }

    #[Test]
    public function it_json_output_is_prettified(): void
    {
        $config = new EmbeddingConfig();
        $json = $config->toJson();

        $this->assertStringContainsString("\n", $json);
        $this->assertStringContainsString('  ', $json);
    }
}
