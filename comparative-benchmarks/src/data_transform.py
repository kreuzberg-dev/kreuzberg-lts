"""Data transformation utilities for converting benchmark results to Polars DataFrames."""

from __future__ import annotations

from typing import TYPE_CHECKING

import msgspec
import polars as pl

from .types import BenchmarkResult, BenchmarkSummary

if TYPE_CHECKING:
    from pathlib import Path


def results_to_dataframe(results: list[BenchmarkResult]) -> pl.DataFrame:
    """Convert list of BenchmarkResult to Polars DataFrame.

    Args:
        results: List of benchmark results

    Returns:
        DataFrame with all benchmark metrics
    """
    if not results:
        return pl.DataFrame()

    data = [msgspec.structs.asdict(result) for result in results]

    df = pl.DataFrame(data)

    if "framework" in df.columns:
        df = df.with_columns(pl.col("framework").cast(pl.String))

    if "file_type" in df.columns:
        df = df.with_columns(pl.col("file_type").cast(pl.String))

    if "category" in df.columns:
        df = df.with_columns(pl.col("category").cast(pl.String))

    if "status" in df.columns:
        df = df.with_columns(pl.col("status").cast(pl.String))

    return df


def summaries_to_dataframe(summaries: list[BenchmarkSummary]) -> pl.DataFrame:
    """Convert list of BenchmarkSummary to Polars DataFrame.

    Args:
        summaries: List of benchmark summaries

    Returns:
        DataFrame with summary metrics
    """
    if not summaries:
        return pl.DataFrame()

    data = [msgspec.structs.asdict(summary) for summary in summaries]

    df = pl.DataFrame(data)

    if "framework" in df.columns:
        df = df.with_columns(pl.col("framework").cast(pl.String))

    if "category" in df.columns:
        df = df.with_columns(pl.col("category").cast(pl.String))

    return df


def load_results_from_json(file_path: Path) -> pl.DataFrame:
    """Load benchmark results from JSON file into DataFrame.

    Args:
        file_path: Path to JSON file containing benchmark results

    Returns:
        DataFrame with benchmark results
    """
    with file_path.open("rb") as f:
        results = msgspec.json.decode(f.read(), type=list[BenchmarkResult])

    return results_to_dataframe(results)


def aggregate_by_framework(df: pl.DataFrame) -> pl.DataFrame:
    """Aggregate benchmark results by framework.

    Args:
        df: DataFrame with individual benchmark results

    Returns:
        DataFrame with aggregated metrics per framework
    """
    return (
        df.group_by("framework")
        .agg(
            pl.col("extraction_time").count().alias("total_files"),
            pl.col("status").filter(pl.col("status") == "success").count().alias("successful_files"),
            pl.col("status").filter(pl.col("status") == "failed").count().alias("failed_files"),
            pl.col("status").filter(pl.col("status") == "timeout").count().alias("timeout_files"),
            pl.col("extraction_time").mean().alias("avg_extraction_time"),
            pl.col("extraction_time").median().alias("median_extraction_time"),
            pl.col("extraction_time").quantile(0.95).alias("p95_extraction_time"),
            pl.col("extraction_time").quantile(0.99).alias("p99_extraction_time"),
            pl.col("extraction_time").std().alias("std_extraction_time"),
            pl.col("extraction_time").min().alias("min_extraction_time"),
            pl.col("extraction_time").max().alias("max_extraction_time"),
            pl.col("peak_memory_mb").mean().alias("avg_peak_memory_mb"),
            pl.col("peak_memory_mb").max().alias("peak_memory_mb"),
            pl.col("avg_cpu_percent").mean().alias("avg_cpu_percent"),
            pl.col("file_size").sum().alias("total_bytes_processed"),
        )
        .with_columns(
            (pl.col("successful_files") / pl.col("total_files")).alias("success_rate"),
            (pl.col("total_files") / pl.col("avg_extraction_time").sum()).alias("files_per_second"),
            (pl.col("total_bytes_processed") / 1024 / 1024 / pl.col("avg_extraction_time").sum()).alias(
                "mb_per_second"
            ),
        )
    )


def aggregate_by_framework_and_format(df: pl.DataFrame) -> pl.DataFrame:
    """Aggregate benchmark results by framework and file type.

    Args:
        df: DataFrame with individual benchmark results

    Returns:
        DataFrame with aggregated metrics per framework and file type
    """
    return (
        df.group_by(["framework", "file_type"])
        .agg(
            pl.col("extraction_time").count().alias("total_files"),
            pl.col("status").filter(pl.col("status") == "SUCCESS").count().alias("successful_files"),
            pl.col("extraction_time").mean().alias("avg_extraction_time"),
            pl.col("extraction_time").median().alias("median_extraction_time"),
            pl.col("peak_memory_mb").mean().alias("avg_peak_memory_mb"),
        )
        .with_columns((pl.col("successful_files") / pl.col("total_files")).alias("success_rate"))
    )


def aggregate_by_framework_and_category(df: pl.DataFrame) -> pl.DataFrame:
    """Aggregate benchmark results by framework and document category.

    Args:
        df: DataFrame with individual benchmark results

    Returns:
        DataFrame with aggregated metrics per framework and category
    """
    return (
        df.group_by(["framework", "category"])
        .agg(
            pl.col("extraction_time").count().alias("total_files"),
            pl.col("status").filter(pl.col("status") == "SUCCESS").count().alias("successful_files"),
            pl.col("extraction_time").mean().alias("avg_extraction_time"),
            pl.col("extraction_time").median().alias("median_extraction_time"),
            pl.col("peak_memory_mb").mean().alias("avg_peak_memory_mb"),
        )
        .with_columns((pl.col("successful_files") / pl.col("total_files")).alias("success_rate"))
    )


def calculate_percentiles(df: pl.DataFrame, column: str, percentiles: list[float]) -> dict[str, float | None]:
    """Calculate multiple percentiles for a column.

    Args:
        df: DataFrame
        column: Column name to calculate percentiles for
        percentiles: List of percentile values (e.g., [0.50, 0.95, 0.99])

    Returns:
        Dictionary mapping percentile names to values
    """
    result = {}
    for p in percentiles:
        percentile_name = f"p{int(p * 100)}_{column}"
        value = df.get_column(column).quantile(p)
        result[percentile_name] = value

    return result


def filter_successful_results(df: pl.DataFrame) -> pl.DataFrame:
    """Filter DataFrame to only successful extractions.

    Args:
        df: DataFrame with benchmark results

    Returns:
        DataFrame containing only successful extractions
    """
    return df.filter(pl.col("status") == "success")


def add_derived_metrics(df: pl.DataFrame) -> pl.DataFrame:
    """Add derived metrics to DataFrame.

    Args:
        df: DataFrame with benchmark results

    Returns:
        DataFrame with additional derived metrics
    """
    return df.with_columns(
        (pl.col("file_size") / 1024 / 1024).alias("file_size_mb"),
        (pl.col("character_count") / pl.col("extraction_time")).alias("chars_per_second"),
        (pl.col("word_count") / pl.col("extraction_time")).alias("words_per_second"),
        (pl.col("peak_memory_mb") / (pl.col("file_size") / 1024 / 1024)).alias("memory_per_mb"),
    )


def export_to_csv(df: pl.DataFrame, output_path: Path) -> Path:
    """Export DataFrame to CSV file.

    Args:
        df: DataFrame to export
        output_path: Path to save CSV file

    Returns:
        Path to exported CSV file
    """
    output_path.parent.mkdir(parents=True, exist_ok=True)
    df.write_csv(str(output_path))
    return output_path


def export_to_json(df: pl.DataFrame, output_path: Path) -> Path:
    """Export DataFrame to JSON file.

    Args:
        df: DataFrame to export
        output_path: Path to save JSON file

    Returns:
        Path to exported JSON file
    """
    output_path.parent.mkdir(parents=True, exist_ok=True)
    df.write_json(str(output_path))
    return output_path


def export_to_parquet(df: pl.DataFrame, output_path: Path) -> Path:
    """Export DataFrame to Parquet file.

    Args:
        df: DataFrame to export
        output_path: Path to save Parquet file

    Returns:
        Path to exported Parquet file
    """
    output_path.parent.mkdir(parents=True, exist_ok=True)
    df.write_parquet(str(output_path))
    return output_path
