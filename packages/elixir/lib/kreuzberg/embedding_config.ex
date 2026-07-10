defmodule Kreuzberg.EmbeddingConfig do
  @moduledoc """
  Configuration for standalone text embedding generation.
  """

  defstruct model: "balanced",
  normalize: true,
  batch_size: nil,
  acceleration: nil

  @type t :: %__MODULE__{
  model: String.t() | map(),
  normalize: boolean(),
  batch_size: pos_integer() | nil,
  acceleration: map() | nil
  }

  @doc """
  Creates a new EmbeddingConfig with default values.
  """
  def new(opts \\ []) do
    struct(__MODULE__, opts)
  end

  @doc """
  Converts the configuration to a map for NIF serialization.
  """
  def to_map(%__MODULE__{} = config) do
    %{
    "model" => normalize_model(config.model),
    "normalize" => config.normalize,
    "batch_size" => config.batch_size,
    "acceleration" => normalize_acceleration(config.acceleration)
    }
  |> Enum.reject(fn {_, v} -> is_nil(v) end)
    |> Map.new()
  end

  defp normalize_model(name) when is_binary(name) do
    %{"type" => "preset", "name" => name}
  end

  defp normalize_model(map) when is_map(map), do: map

  defp normalize_acceleration(nil), do: nil

  defp normalize_acceleration(accel_config) when is_map(accel_config) do
    normalized =
    accel_config
    |> Enum.reduce(%{}, fn
      {key, value}, acc when is_binary(key) -> Map.put(acc, key, value)
      {key, value}, acc -> Map.put(acc, Atom.to_string(key), value)
    end)

    normalized =
    if Map.has_key?(normalized, "provider") do
      normalized
    else
      Map.put(normalized, "provider", "auto")
    end

    if Map.has_key?(normalized, "device_id") do
      normalized
    else
      Map.put(normalized, "device_id", 0)
    end
  end

  defp normalize_acceleration(other), do: other
end
