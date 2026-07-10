using System;
using System.Runtime.CompilerServices;
using Xunit;

[assembly: CollectionBehavior(DisableTestParallelization = true)]

namespace Kreuzberg.Tests;

/// <summary>
/// Module initializer that runs before any tests execute.
/// This ensures Pdfium is initialized exactly once per test run.
/// </summary>
internal static class PdfiumModuleInitializer
{
    [ModuleInitializer]
    public static void Initialize()
    {
        PdfiumInitializer.Initialize();
    }
}

/// <summary>
/// Static initializer for Pdfium library.
/// Ensures initialization happens exactly once per test run.
/// </summary>
internal static class PdfiumInitializer
{
    private static volatile bool s_initialized = false;
    private static volatile bool s_cleanupStarted = false;
    private static readonly object s_lock = new();

    public static void Initialize()
    {
        if (s_initialized)
        return;

        lock (s_lock)
        {
            if (s_initialized)
            return;

            try
            {
                System.Console.WriteLine("[Test Init] Loading native library...");

                NativeTestHelper.EnsureNativeLibraryLoaded();

                // NOTE: We intentionally do NOT register ProcessExit/DomainUnload handlers

                System.Console.WriteLine("[Test Init] Native library loaded. Pdfium will initialize lazily on first use.");
                s_initialized = true;
            }
            catch (Exception ex)
            {
                System.Console.WriteLine($"[Test Init] Warning: {ex.Message}");
                s_initialized = true;
            }
        }
    }

    /// <summary>
    /// Marks that cleanup has started to prevent further FFI calls.
    /// This is called by test cleanup to signal that the process is shutting down.
    /// </summary>
    internal static void MarkCleanupStarted()
    {
        s_cleanupStarted = true;
    }

    /// <summary>
    /// Returns true if cleanup has been started and FFI calls should be avoided.
    /// </summary>
    internal static bool IsCleanupStarted => s_cleanupStarted;
}
