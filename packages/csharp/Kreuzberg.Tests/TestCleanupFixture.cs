using System;
using Xunit;

namespace Kreuzberg.Tests;

/// <summary>
/// Test fixture that ensures cleanup of registered callbacks after each test method.
/// This prevents resource leaks from accumulated GCHandles for post-processors,
/// validators, and OCR backends.
/// </summary>
public class TestCleanupFixture : IDisposable
{
    public TestCleanupFixture()
    {
        NativeTestHelper.EnsureNativeLibraryLoaded();
    }

    public void Dispose()
    {
        CleanupAllRegistrations();
    }

    private static void CleanupAllRegistrations()
    {
        try
        {
            KreuzbergClient.ClearPostProcessors();
        }
        catch
        {
        }

        try
        {
            KreuzbergClient.ClearValidators();
        }
        catch
        {
        }

        try
        {
            KreuzbergClient.ClearOcrBackends();
        }
        catch
        {
        }
    }
}
