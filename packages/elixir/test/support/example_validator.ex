defmodule Kreuzberg.Test.ExampleValidator do
  @moduledoc """
  Example validator plugin for non-empty content validation.

  This validator demonstrates how to implement a custom validator that checks
  whether extraction results contain non-empty content. It runs with high priority
  to catch empty content early in the validation pipeline.

  ## Validation Priority

  Runs with priority 100 (high priority), ensuring this validation executes
  before lower priority validators.

  ## Behavior

  - Checks if content field exists and is not empty
  - Skips validation if result is not a valid map
  - Returns descriptive error messages on failure

  ## Example

      # Validation passes
      result = %{"content" => "Some text", "mime_type" => "text/plain"}
      Kreuzberg.Test.ExampleValidator.validate(result)
      # Returns: :ok

      # Validation fails
      result = %{"content" => "", "mime_type" => "text/plain"}
      Kreuzberg.Test.ExampleValidator.validate(result)
      # Returns: {:error, "Extraction result contains empty content"}

      # Validation skipped
      Kreuzberg.Test.ExampleValidator.should_validate?(%{})
      # Returns: false
  """

  @behaviour Kreuzberg.Plugin.Validator

  @impl true
  def name do
    "non_empty"
  end

  @impl true
  def version do
    "1.0.0"
  end

  @impl true
  def priority do
    100
  end

  @impl true
  def initialize do
    :ok
  end

  @impl true
  def shutdown do
    :ok
  end

  @impl true
  def validate(result) do
    case result do
      %{"content" => content} when is_binary(content) ->
      if String.trim(content) == "" do
        {:error, "Extraction result contains empty content"}
      else
        :ok
      end

      %{:content => content} when is_binary(content) ->
      if String.trim(content) == "" do
        {:error, "Extraction result contains empty content"}
      else
        :ok
      end

      _ ->
      {:error, "Result must contain a content field with string value"}
    end
  end

  @impl true
  def should_validate?(result) do
    is_map(result) and
    (Map.has_key?(result, "content") or Map.has_key?(result, :content))
  end
end
