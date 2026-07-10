using System;
using Xunit;

namespace Kreuzberg.Tests;

/// <summary>
/// Base class for all test classes that ensures proper cleanup of registered callbacks.
/// This prevents resource leaks from accumulated GCHandles.
/// </summary>
public abstract class TestBase : IDisposable
{
    protected TestBase()
    {
        NativeTestHelper.EnsureNativeLibraryLoaded();
    }

    public virtual void Dispose()
    {
        CleanupRegistrations();
        GC.SuppressFinalize(this);
    }

    private static void CleanupRegistrations()
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
