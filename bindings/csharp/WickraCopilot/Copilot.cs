using System.Runtime.InteropServices;
using System.Text;

namespace Wickra.Copilot;

/// <summary>
/// A copilot instance driven by JSON commands, over the Wickra C ABI. Build one
/// from a spec JSON, drive it with command JSON and read back the response JSON —
/// the same protocol as the CLI and every other binding. Only the deterministic
/// core is exposed; the LLM adapter is never reachable over the C ABI.
/// </summary>
public sealed class Copilot : IDisposable
{
    private readonly CopilotHandle _handle;

    /// <summary>Build a copilot from a spec JSON string.</summary>
    /// <exception cref="ArgumentException">The spec is null/invalid.</exception>
    public Copilot(string specJson)
    {
        IntPtr ptr = Native.wickra_copilot_new(Utf8(specJson));
        if (ptr == IntPtr.Zero)
        {
            throw new ArgumentException("wickra-copilot: invalid spec", nameof(specJson));
        }
        _handle = new CopilotHandle(ptr);
    }

    /// <summary>Apply a command JSON and return the response JSON.</summary>
    /// <remarks>
    /// Uses the C ABI's length-out protocol: a first call learns the length, then
    /// the response is read into a caller-owned buffer. Domain errors (a bad spec,
    /// an unknown command) come back in-band as <c>{"ok":false,...}</c> JSON, not
    /// as an exception.
    /// </remarks>
    /// <exception cref="InvalidOperationException">A required argument was unusable or a panic was caught.</exception>
    public string Command(string cmdJson)
    {
        ObjectDisposedException.ThrowIf(_handle.IsInvalid, this);

        byte[] cmd = Utf8(cmdJson);
        IntPtr h = _handle.DangerousGetHandle();
        int n = Native.wickra_copilot_command(h, cmd, null, 0);
        if (n < 0)
        {
            throw new InvalidOperationException($"wickra-copilot: command failed (code {n})");
        }
        var buf = new byte[n + 1];
        Native.wickra_copilot_command(h, cmd, buf, (nuint)buf.Length);
        return Encoding.UTF8.GetString(buf, 0, n);
    }

    /// <summary>The library version.</summary>
    public static string Version() =>
        Marshal.PtrToStringUTF8(Native.wickra_copilot_version()) ?? string.Empty;

    /// <summary>Free the native copilot handle.</summary>
    public void Dispose() => _handle.Dispose();

    /// <summary>Encode a string as NUL-terminated UTF-8 for the C ABI.</summary>
    private static byte[] Utf8(string s)
    {
        int len = Encoding.UTF8.GetByteCount(s);
        var buf = new byte[len + 1];
        Encoding.UTF8.GetBytes(s, 0, s.Length, buf, 0);
        return buf;
    }
}

/// <summary>A safe handle owning a native copilot pointer.</summary>
internal sealed class CopilotHandle : SafeHandle
{
    public CopilotHandle(IntPtr handle)
        : base(IntPtr.Zero, ownsHandle: true) => SetHandle(handle);

    public override bool IsInvalid => handle == IntPtr.Zero;

    protected override bool ReleaseHandle()
    {
        Native.wickra_copilot_free(handle);
        return true;
    }
}
