using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace Wickra.Copilot;

/// <summary>Raw P/Invoke surface for the wickra-copilot C ABI.</summary>
internal static partial class Native
{
    internal const string Lib = "wickra_copilot";

    /// <summary>Build a copilot from a spec JSON (NUL-terminated UTF-8). Null on error.</summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial IntPtr wickra_copilot_new(byte[] specUtf8);

    /// <summary>Free a copilot handle.</summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial void wickra_copilot_free(IntPtr handle);

    /// <summary>
    /// Apply a command JSON (NUL-terminated UTF-8), writing the response into a
    /// caller-owned buffer. Returns the response length, or a negative error code.
    /// </summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial int wickra_copilot_command(IntPtr handle, byte[] cmdUtf8, byte[]? outBuf, nuint cap);

    /// <summary>The library version as a static NUL-terminated string.</summary>
    [LibraryImport(Lib)]
    [UnmanagedCallConv(CallConvs = [typeof(CallConvCdecl)])]
    internal static partial IntPtr wickra_copilot_version();
}
