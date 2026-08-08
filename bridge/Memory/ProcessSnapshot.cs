using System.ComponentModel;
using System.Diagnostics;
using Microsoft.Win32.SafeHandles;

namespace FmDataBridge.Memory;

/// <summary>Creates a one-shot frozen reader for a failed live scan.</summary>
internal interface IProcessSnapshotFactory
{
    ProcessSnapshotCaptureResult TryCapture();
}

/// <summary>Owns a VA-clone reader until the retry finishes.</summary>
internal interface IProcessSnapshot : IDisposable
{
    IMemoryReader Reader { get; }
}

internal sealed class ProcessSnapshotCaptureResult
{
    private ProcessSnapshotCaptureResult(
        IProcessSnapshot? snapshot,
        string? failureReason,
        long captureMilliseconds)
    {
        Snapshot = snapshot;
        FailureReason = failureReason;
        CaptureMilliseconds = captureMilliseconds;
    }

    public IProcessSnapshot? Snapshot { get; }

    public string? FailureReason { get; }

    public long CaptureMilliseconds { get; }

    public bool IsSuccess => Snapshot is not null;

    public static ProcessSnapshotCaptureResult Succeeded(
        IProcessSnapshot snapshot,
        long captureMilliseconds = 0)
    {
        ArgumentNullException.ThrowIfNull(snapshot);
        return new ProcessSnapshotCaptureResult(snapshot, failureReason: null, captureMilliseconds);
    }

    public static ProcessSnapshotCaptureResult Failed(
        string failureReason,
        long captureMilliseconds = 0)
    {
        if (string.IsNullOrEmpty(failureReason))
        {
            throw new ArgumentException("A snapshot failure reason is required.", nameof(failureReason));
        }

        return new ProcessSnapshotCaptureResult(snapshot: null, failureReason, captureMilliseconds);
    }
}

internal static class ProcessSnapshotPolicy
{
    internal const ulong MinimumAvailableCommitBytes = MemoryPressurePolicy.LowMemoryThresholdBytes;

    internal static bool HasSufficientAvailableCommit(SystemMemoryStatus memoryStatus) =>
        memoryStatus.IsKnown
        && memoryStatus.AvailableCommitBytes >= MinimumAvailableCommitBytes;
}

internal sealed class WindowsProcessSnapshotFactory : IProcessSnapshotFactory
{
    private const uint PssCaptureVaClone = 0x00000001;
    private const uint PssQueryVaCloneInformation = 1;

    public ProcessSnapshotCaptureResult TryCapture()
    {
        if (!OperatingSystem.IsWindows())
        {
            return ProcessSnapshotCaptureResult.Failed(
                "PSS VA-clone snapshots are supported only on Windows");
        }

        var stopwatch = Stopwatch.StartNew();
        var snapshotHandle = IntPtr.Zero;
        var cloneHandle = IntPtr.Zero;
        try
        {
            var captureResult = NativeMethods.PssCaptureSnapshot(
                NativeMethods.GetCurrentProcess(),
                PssCaptureVaClone,
                threadContextFlags: 0,
                out snapshotHandle);
            if (captureResult != 0)
            {
                return ProcessSnapshotCaptureResult.Failed(
                    FormatNativeFailure("capture", captureResult),
                    stopwatch.ElapsedMilliseconds);
            }

            var queryResult = NativeMethods.PssQuerySnapshot(
                snapshotHandle,
                PssQueryVaCloneInformation,
                out var cloneInformation,
                (uint)System.Runtime.InteropServices.Marshal.SizeOf<NativeMethods.PssVaCloneInformation>());
            cloneHandle = cloneInformation.VaCloneHandle;
            if (queryResult != 0)
            {
                return ProcessSnapshotCaptureResult.Failed(
                    FormatNativeFailure("query", queryResult),
                    stopwatch.ElapsedMilliseconds);
            }

            if (cloneHandle == IntPtr.Zero)
            {
                return ProcessSnapshotCaptureResult.Failed(
                    "PSS query returned an empty VA-clone handle",
                    stopwatch.ElapsedMilliseconds);
            }

            var snapshot = new WindowsProcessSnapshot(snapshotHandle, cloneHandle);
            snapshotHandle = IntPtr.Zero;
            cloneHandle = IntPtr.Zero;
            return ProcessSnapshotCaptureResult.Succeeded(snapshot, stopwatch.ElapsedMilliseconds);
        }
        catch (Exception exception)
        {
            return ProcessSnapshotCaptureResult.Failed(
                $"PSS VA-clone snapshot threw {exception.GetType().Name}: {exception.Message}",
                stopwatch.ElapsedMilliseconds);
        }
        finally
        {
            if (cloneHandle != IntPtr.Zero)
            {
                try
                {
                    _ = NativeMethods.CloseHandle(cloneHandle);
                }
                catch
                {
                    // The capture failure is already reported to the caller.
                }
            }

            if (snapshotHandle != IntPtr.Zero)
            {
                try
                {
                    _ = NativeMethods.PssFreeSnapshot(NativeMethods.GetCurrentProcess(), snapshotHandle);
                }
                catch
                {
                    // The capture failure is already reported to the caller.
                }
            }
        }
    }

    private static string FormatNativeFailure(string operation, uint errorCode) =>
        $"PSS {operation} failed (Win32 {errorCode}: {new Win32Exception((int)errorCode).Message})";
}

internal sealed class WindowsProcessSnapshot : IProcessSnapshot
{
    private readonly PssSnapshotHandle _snapshotHandle;

    internal WindowsProcessSnapshot(IntPtr snapshotHandle, IntPtr cloneHandle)
    {
        var clone = new VaCloneHandle(cloneHandle);
        _snapshotHandle = new PssSnapshotHandle(snapshotHandle, clone);
        Reader = new WindowsMemoryReader(_snapshotHandle.CloneHandle, "snapshot-va-clone");
    }

    public IMemoryReader Reader { get; }

    public void Dispose() => _snapshotHandle.Dispose();

    private sealed class PssSnapshotHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        private readonly VaCloneHandle _cloneHandle;

        internal PssSnapshotHandle(IntPtr snapshotHandle, VaCloneHandle cloneHandle)
            : base(ownsHandle: true)
        {
            _cloneHandle = cloneHandle;
            SetHandle(snapshotHandle);
        }

        internal IntPtr CloneHandle => _cloneHandle.DangerousGetHandle();

        protected override bool ReleaseHandle()
        {
            try
            {
                _cloneHandle.Dispose();
                return NativeMethods.PssFreeSnapshot(NativeMethods.GetCurrentProcess(), handle) == 0;
            }
            catch
            {
                return false;
            }
        }
    }

    private sealed class VaCloneHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        internal VaCloneHandle(IntPtr cloneHandle)
            : base(ownsHandle: true)
        {
            SetHandle(cloneHandle);
        }

        protected override bool ReleaseHandle()
        {
            try
            {
                return NativeMethods.CloseHandle(handle);
            }
            catch
            {
                return false;
            }
        }
    }
}
