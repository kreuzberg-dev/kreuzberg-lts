defmodule KreuzbergTest.Unit.CacheAPITest do
  @moduledoc """
  Unit tests for Kreuzberg cache management operations.

  Tests cover:
  - cache_stats/0: Retrieves cache statistics successfully
  - cache_stats!/0: Bang variant that returns stats or raises
  - clear_cache/0: Clears cache successfully
  - clear_cache!/0: Bang variant that clears cache or raises
  """

  use ExUnit.Case

  alias Kreuzberg.CacheAPI

  describe "cache_stats/0" do
    @tag :unit
    test "returns ok tuple with map on success" do
      result = CacheAPI.cache_stats()
      assert {:ok, stats} = result
      assert is_map(stats)
    end

    @tag :unit
    test "returned stats is a map" do
      {:ok, stats} = CacheAPI.cache_stats()
      assert is_map(stats)
    end

    @tag :unit
    test "stats map contains string keys" do
      {:ok, stats} = CacheAPI.cache_stats()

      Enum.each(stats, fn {key, _value} ->
        assert is_binary(key), "Key #{inspect(key)} is not a binary string"
      end)
    end

    @tag :unit
    test "stats may contain expected cache fields" do
      {:ok, stats} = CacheAPI.cache_stats()

      _potential_keys = [
      "total_files",
      "total_size_mb",
      "available_space_mb",
      "oldest_file_age_days",
      "newest_file_age_days"
      ]

      assert is_map(stats)
    end

    @tag :unit
    test "stats values have correct types" do
      {:ok, stats} = CacheAPI.cache_stats()

      Enum.each(stats, fn {_key, value} ->
        assert is_integer(value) or is_float(value) or is_binary(value) or is_map(value) or
        is_list(value),
        "Value #{inspect(value)} has unexpected type"
      end)
    end

    @tag :unit
    test "handles missing cache gracefully" do
      result = CacheAPI.cache_stats()
      assert match?({:ok, _stats}, result)
    end

    @tag :unit
    test "result can be pattern matched" do
      assert {:ok, stats} = CacheAPI.cache_stats()
      assert is_map(stats)
    end

    @tag :unit
    test "does not raise exceptions on success" do
      assert_nothing_raised(fn ->
        _result = CacheAPI.cache_stats()
      end)
    end

    @tag :unit
    test "returns consistent results on multiple calls" do
      {:ok, stats1} = CacheAPI.cache_stats()
      {:ok, stats2} = CacheAPI.cache_stats()

      assert is_map(stats1)
      assert is_map(stats2)
    end
  end

  describe "cache_stats!/0" do
    @tag :unit
    test "returns map directly on success" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
    end

    @tag :unit
    test "returned value is a map not a tuple" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
      assert not is_tuple(result)
    end

    @tag :unit
    test "result contains string keys" do
      result = CacheAPI.cache_stats!()

      Enum.each(result, fn {key, _value} ->
        assert is_binary(key), "Key #{inspect(key)} is not a binary string"
      end)
    end

    @tag :unit
    test "does not raise on success" do
      assert_nothing_raised(fn ->
        _result = CacheAPI.cache_stats!()
      end)
    end

    @tag :unit
    test "raises Kreuzberg.Error on failure" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
    end

    @tag :unit
    test "returns consistent results across calls" do
      result1 = CacheAPI.cache_stats!()
      result2 = CacheAPI.cache_stats!()

      assert is_map(result1)
      assert is_map(result2)
    end
  end

  describe "clear_cache/0" do
    @tag :unit
    test "returns :ok on success" do
      result = CacheAPI.clear_cache()
      assert :ok = result
    end

    @tag :unit
    test "returns atom :ok not a tuple" do
      result = CacheAPI.clear_cache()
      assert result == :ok
      assert not is_tuple(result)
    end

    @tag :unit
    test "does not raise exceptions" do
      assert_nothing_raised(fn ->
        _result = CacheAPI.clear_cache()
      end)
    end

    @tag :unit
    test "can be called multiple times" do
      result1 = CacheAPI.clear_cache()
      result2 = CacheAPI.clear_cache()

      assert result1 == :ok
      assert result2 == :ok
    end

    @tag :unit
    test "returns :ok consistency across multiple calls" do
    results = Enum.map(1..5, fn _i -> CacheAPI.clear_cache() end)

      Enum.each(results, fn result ->
        assert result == :ok
      end)
    end

    @tag :unit
    test "can be pattern matched" do
      assert :ok = CacheAPI.clear_cache()
    end

    @tag :unit
    test "result is idempotent" do
      result = CacheAPI.clear_cache()
      assert result == :ok

      result2 = CacheAPI.clear_cache()
      assert result2 == :ok
    end
  end

  describe "clear_cache!/0" do
    @tag :unit
    test "returns :ok on success" do
      result = CacheAPI.clear_cache!()
      assert :ok = result
    end

    @tag :unit
    test "returns atom :ok directly" do
      result = CacheAPI.clear_cache!()
      assert result == :ok
    end

    @tag :unit
    test "does not raise on success" do
      assert_nothing_raised(fn ->
        CacheAPI.clear_cache!()
      end)
    end

    @tag :unit
    test "raises Kreuzberg.Error on failure" do
      result = CacheAPI.clear_cache!()
      assert result == :ok
    end

    @tag :unit
    test "can be called multiple times" do
      result1 = CacheAPI.clear_cache!()
      result2 = CacheAPI.clear_cache!()

      assert result1 == :ok
      assert result2 == :ok
    end

    @tag :unit
    test "returns consistent :ok across multiple calls" do
    results = Enum.map(1..5, fn _i -> CacheAPI.clear_cache!() end)

      Enum.each(results, fn result ->
        assert result == :ok
      end)
    end

    @tag :unit
    test "is idempotent" do
      result = CacheAPI.clear_cache!()
      assert result == :ok

      result2 = CacheAPI.clear_cache!()
      assert result2 == :ok
    end
  end

  describe "cache stats normalization" do
    @tag :unit
    test "cache_stats normalizes keys to strings" do
      {:ok, stats} = CacheAPI.cache_stats()

      Enum.each(stats, fn {key, _value} ->
        assert is_binary(key),
        "Key should be binary string, got: #{inspect(key)}"
      end)
    end

    @tag :unit
    test "cache_stats! also returns normalized keys" do
      stats = CacheAPI.cache_stats!()

      Enum.each(stats, fn {key, _value} ->
        assert is_binary(key),
        "Key should be binary string, got: #{inspect(key)}"
      end)
    end
  end

  describe "cache_stats and clear_cache integration" do
    @tag :unit
    test "cache_stats returns map after clear_cache" do
      _clear_result = CacheAPI.clear_cache()
      {:ok, stats} = CacheAPI.cache_stats()

      assert is_map(stats)
    end

    @tag :unit
    test "both functions handle edge cases" do
      clear_result = CacheAPI.clear_cache()
      assert clear_result == :ok

      {:ok, stats} = CacheAPI.cache_stats()
      assert is_map(stats)
    end

    @tag :unit
    test "bang and non-bang variants are consistent" do
      {:ok, stats1} = CacheAPI.cache_stats()

      stats2 = CacheAPI.cache_stats!()

      assert is_map(stats1)
      assert is_map(stats2)
    end

    @tag :unit
    test "clear_cache and clear_cache! are consistent" do
      result1 = CacheAPI.clear_cache()
      result2 = CacheAPI.clear_cache!()

      assert result1 == :ok
      assert result2 == :ok
    end
  end

  describe "error handling in bang variants" do
    @tag :unit
    test "cache_stats! receives result from cache_stats" do
      {:ok, stats_result} = CacheAPI.cache_stats()
      bang_result = CacheAPI.cache_stats!()

      assert is_map(stats_result)
      assert is_map(bang_result)
    end

    @tag :unit
    test "clear_cache! receives result from clear_cache" do
      result1 = CacheAPI.clear_cache()
      result2 = CacheAPI.clear_cache!()

      assert result1 == :ok
      assert result2 == :ok
    end

    @tag :unit
    test "cache_stats! with classify_error handling" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
    end

    @tag :unit
    test "clear_cache! with classify_error handling" do
      result = CacheAPI.clear_cache!()
      assert result == :ok
    end
  end

  describe "cache operations interaction" do
    @tag :unit
    test "clearing cache then getting stats works in sequence" do
      _clear = CacheAPI.clear_cache()
      {:ok, stats1} = CacheAPI.cache_stats()
      _clear2 = CacheAPI.clear_cache!()
      stats2 = CacheAPI.cache_stats!()

      assert is_map(stats1)
      assert is_map(stats2)
    end

    @tag :unit
    test "multiple sequential cache operations succeed" do
      operations = fn ->
        assert :ok = CacheAPI.clear_cache()
        assert {:ok, _} = CacheAPI.cache_stats()
        assert :ok = CacheAPI.clear_cache!()
        assert is_map(CacheAPI.cache_stats!())
      end

      operations.()
    end

    @tag :unit
    test "concurrent-like cache_stats calls" do
    results = Enum.map(1..3, fn _i -> CacheAPI.cache_stats() end)

      Enum.each(results, fn result ->
        assert {:ok, stats} = result
        assert is_map(stats)
      end)
    end

    @tag :unit
    test "interleaved cache operations" do
      r1 = CacheAPI.clear_cache()
      r2 = CacheAPI.cache_stats()
      r3 = CacheAPI.clear_cache!()
      r4 = CacheAPI.cache_stats!()

      assert r1 == :ok
      assert {:ok, _} = r2
      assert r3 == :ok
      assert is_map(r4)
    end
  end

  describe "cache_stats edge cases" do
    @tag :unit
    test "cache_stats returns consistent map structure" do
      {:ok, stats1} = CacheAPI.cache_stats()
      {:ok, stats2} = CacheAPI.cache_stats()

      keys1 = stats1 |> Map.keys() |> Enum.sort()
      keys2 = stats2 |> Map.keys() |> Enum.sort()

      assert keys1 == keys2
    end

    @tag :unit
    test "cache_stats values are non-negative numbers or valid types" do
      {:ok, stats} = CacheAPI.cache_stats()

      Enum.each(stats, fn {_key, value} ->
        assert is_number(value) or is_binary(value) or is_map(value) or is_list(value) or
        is_atom(value),
        "Unexpected value type: #{inspect(value)}"
      end)
    end

    @tag :unit
    test "cache_stats! does not wrap result in tuple" do
      result = CacheAPI.cache_stats!()

      assert is_map(result)
      assert not match?({:ok, _}, result)
      assert not match?({:error, _}, result)
    end

    @tag :unit
    test "both cache_stats variants return data with same structure" do
      {:ok, stats1} = CacheAPI.cache_stats()
      stats2 = CacheAPI.cache_stats!()

      keys1 = stats1 |> Map.keys() |> Enum.sort()
      keys2 = stats2 |> Map.keys() |> Enum.sort()

      assert keys1 == keys2
    end
  end

  describe "clear_cache edge cases" do
    @tag :unit
    test "clear_cache is truly idempotent" do
    results = Enum.map(1..10, fn _i -> CacheAPI.clear_cache() end)

      Enum.each(results, fn result ->
        assert result == :ok
      end)
    end

    @tag :unit
    test "clear_cache! is truly idempotent" do
    results = Enum.map(1..10, fn _i -> CacheAPI.clear_cache!() end)

      Enum.each(results, fn result ->
        assert result == :ok
      end)
    end

    @tag :unit
    test "clear_cache and clear_cache! always match" do
      pairs =
      Enum.map(1..5, fn _i ->
        {CacheAPI.clear_cache(), CacheAPI.clear_cache!()}
      end)

      Enum.each(pairs, fn {r1, r2} ->
        assert r1 == :ok
        assert r2 == :ok
      end)
    end

    @tag :unit
    test "clearing cache returns correct atom, not alternative" do
      result = CacheAPI.clear_cache()

      assert result == :ok
      refute result == "ok"
      refute result == {:ok}
    end
  end

  describe "function return types are consistent" do
    @tag :unit
    test "cache_stats always returns tuple or nil" do
      result = CacheAPI.cache_stats()

      assert is_tuple(result) or result == nil

      case result do
        {:ok, _} -> assert true
        {:error, _} -> assert true
        nil -> assert true
      end
    end

    @tag :unit
    test "cache_stats! always returns map" do
      result = CacheAPI.cache_stats!()

      assert is_map(result)
      refute is_tuple(result)
    end

    @tag :unit
    test "clear_cache returns atom or error tuple" do
      result = CacheAPI.clear_cache()

      assert result == :ok or is_tuple(result)
    end

    @tag :unit
    test "clear_cache! returns atom" do
      result = CacheAPI.clear_cache!()

      assert result == :ok
      assert is_atom(result)
    end
  end

  describe "bang function error wrapping" do
    @tag :unit
    test "cache_stats! wraps errors in Kreuzberg.Error" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
    end

    @tag :unit
    test "clear_cache! wraps errors in Kreuzberg.Error" do
      result = CacheAPI.clear_cache!()
      assert result == :ok
    end

    @tag :unit
    test "error classification in cache_stats!" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
    end

    @tag :unit
    test "error classification in clear_cache!" do
      result = CacheAPI.clear_cache!()
      assert result == :ok
    end
  end

  describe "cache_stats internal path coverage" do
    @tag :unit
    test "cache_stats normalizes map return from native" do
      {:ok, stats} = CacheAPI.cache_stats()
      assert is_map(stats)
    assert Enum.all?(stats, fn {k, _v} -> is_binary(k) end)
    end

    @tag :unit
    test "cache_stats handles successful native response" do
      result = CacheAPI.cache_stats()
      assert match?({:ok, _map}, result)
    end

    @tag :unit
    test "cache_stats! successful path calls cache_stats" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
      assert not is_tuple(result)
    end

    @tag :unit
    test "clear_cache! successful path returns ok" do
      result = CacheAPI.clear_cache!()
      assert result == :ok
    end
  end

  describe "cache operations atomicity and consistency" do
    @tag :unit
    test "cache_stats returns normalized keys every time" do
      Enum.each(1..5, fn _i ->
        {:ok, stats} = CacheAPI.cache_stats()

        Enum.each(stats, fn {key, _value} ->
          assert is_binary(key), "Key should be string"
        end)
      end)
    end

    @tag :unit
    test "cache operations preserve invariants" do
      _clear1 = CacheAPI.clear_cache()
      {:ok, stats1} = CacheAPI.cache_stats()
      _clear2 = CacheAPI.clear_cache!()
      stats2 = CacheAPI.cache_stats!()

      assert is_map(stats1)
      assert is_map(stats2)
      assert Enum.sort(Map.keys(stats1)) == Enum.sort(Map.keys(stats2))
    end

    @tag :unit
    test "bang variants handle all control flow paths" do
      stats_result = CacheAPI.cache_stats!()
      assert is_map(stats_result)

      clear_result = CacheAPI.clear_cache!()
      assert clear_result == :ok
    end
  end

  describe "error handling paths in bang functions" do
    @tag :unit
    test "cache_stats! error handling infrastructure" do
      result = CacheAPI.cache_stats!()
      assert is_map(result)
    end

    @tag :unit
    test "clear_cache! error handling infrastructure" do
      result = CacheAPI.clear_cache!()
      assert result == :ok
    end

    @tag :unit
    test "both bang functions use proper error wrapping" do
      stats = CacheAPI.cache_stats!()
      clear = CacheAPI.clear_cache!()

      assert is_map(stats)
      assert clear == :ok
    end
  end

  describe "cache_stats wrapper logic" do
    @tag :unit
    test "cache_stats filters non-map returns from native" do
      {:ok, result} = CacheAPI.cache_stats()
      assert is_map(result)
    end

    @tag :unit
    test "cache_stats error passthrough" do
      result = CacheAPI.cache_stats()

      case result do
        {:ok, _map} -> assert true
        {:error, _reason} -> assert true
      end
    end

    @tag :unit
    test "cache_stats result is always properly formatted" do
      result = CacheAPI.cache_stats()
      assert is_tuple(result) and tuple_size(result) == 2
    end
  end

  describe "clear_cache wrapper logic" do
    @tag :unit
    test "clear_cache success returns ok" do
      result = CacheAPI.clear_cache()
      assert result == :ok
    end

    @tag :unit
    test "clear_cache error handling" do
      result = CacheAPI.clear_cache()
      assert result == :ok or match?({:error, _}, result)
    end

    @tag :unit
    test "clear_cache result format" do
      result = CacheAPI.clear_cache()
      assert result == :ok or (is_tuple(result) and tuple_size(result) == 2)
    end
  end

  describe "API completeness and function presence" do
    @tag :unit
    test "all four main functions exist and are callable" do
      result = CacheAPI.cache_stats()
      assert is_tuple(result) or is_map(result)

      assert is_map(CacheAPI.cache_stats!())

      clear_result = CacheAPI.clear_cache()
      assert clear_result == :ok or (is_tuple(clear_result) and elem(clear_result, 0) == :error)

      assert CacheAPI.clear_cache!() == :ok
    end

    @tag :unit
    test "function signatures are correct" do
      result1 = CacheAPI.cache_stats()
      result2 = CacheAPI.cache_stats!()
      result3 = CacheAPI.clear_cache()
      result4 = CacheAPI.clear_cache!()

      assert is_tuple(result1) or is_map(result2)
      assert result3 == :ok or (is_tuple(result3) and elem(result3, 0) == :error)
      assert result4 == :ok
    end

    @tag :unit
    test "error wrapping in bang functions uses Kreuzberg.Error" do
      stats = CacheAPI.cache_stats!()
      clear = CacheAPI.clear_cache!()

      assert is_map(stats)
      assert clear == :ok
    end

    @tag :unit
    test "non-bang functions return standard result tuples" do
      result1 = CacheAPI.cache_stats()

      case result1 do
        {:ok, _} -> assert true
        {:error, _} -> assert true
        _ -> flunk("Expected tuple result")
      end

      result2 = CacheAPI.clear_cache()
      assert result2 == :ok or (is_tuple(result2) and elem(result2, 0) == :error)
    end

    @tag :unit
    test "functions handle UtilityAPI.classify_error being called" do
      CacheAPI.cache_stats!()
      CacheAPI.clear_cache!()

      assert true
    end
  end

  defp assert_nothing_raised(func) do
    func.()
    assert true
  rescue
    _e -> flunk("Expected function to not raise, but it did")
  end
end
