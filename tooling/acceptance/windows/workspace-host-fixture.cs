// Synthetic windows only. Never accepts an external PID or HWND.
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
using System.Windows.Forms;

internal static class WorkspaceHostFixture
{
    const int Style = -16;
    const long Child = 0x40000000L, Popup = 0x80000000L;
    const uint FrameChanged = 0x20, NoActivate = 0x10, NoZOrder = 4;
    [DllImport("user32.dll", SetLastError = true)] static extern IntPtr SetParent(IntPtr child, IntPtr parent);
    [DllImport("user32.dll")] static extern IntPtr GetParent(IntPtr window);
    [DllImport("user32.dll")] static extern IntPtr GetWindowLongPtrW(IntPtr window, int index);
    [DllImport("user32.dll", SetLastError = true)] static extern IntPtr SetWindowLongPtrW(IntPtr window, int index, IntPtr value);
    [DllImport("kernel32.dll")] static extern void SetLastError(uint value);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint pid);
    [DllImport("user32.dll")] static extern bool IsWindow(IntPtr window);
    [DllImport("user32.dll", SetLastError = true)] static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr context);
    [DllImport("user32.dll")] static extern IntPtr GetWindowDpiAwarenessContext(IntPtr window);
    [DllImport("user32.dll")] static extern bool AreDpiAwarenessContextsEqual(IntPtr first, IntPtr second);
    [StructLayout(LayoutKind.Sequential)] struct Rect { public int Left, Top, Right, Bottom; }

    static void Require(bool condition, string message) { if (!condition) throw new InvalidOperationException(message); }
    static void CheckOwned(Process process, DateTime started, IntPtr window)
    {
        uint pid;
        Require(!process.HasExited && process.StartTime.ToUniversalTime() == started, "Fixture process identity changed");
        GetWindowThreadProcessId(window, out pid);
        Require(IsWindow(window) && pid == process.Id, "Fixture window identity changed");
    }
    static void SetStyle(IntPtr window, long value)
    {
        SetLastError(0);
        IntPtr previous = SetWindowLongPtrW(window, Style, new IntPtr(value));
        int error = Marshal.GetLastWin32Error();
        if (previous == IntPtr.Zero && error != 0) throw new Win32Exception(error);
    }
    static void Reparent(IntPtr window, IntPtr parent)
    {
        SetLastError(0);
        IntPtr previous = SetParent(window, parent);
        int error = Marshal.GetLastWin32Error();
        if (previous == IntPtr.Zero && error != 0) throw new Win32Exception(error);
    }
    static void Place(IntPtr window, int x, int y, int width, int height)
    {
        if (!SetWindowPos(window, IntPtr.Zero, x, y, width, height, FrameChanged | NoActivate | NoZOrder))
            throw new Win32Exception(Marshal.GetLastWin32Error());
    }

    [STAThread] static int Main(string[] args)
    {
        try { return Run(args); }
        catch (Exception ex)
        {
            if (args.Length == 2) File.WriteAllText(args[1] + ".error", ex.ToString());
            return 1;
        }
    }
    static int Run(string[] args)
    {
        // Match the manager's PMv2 context. Mixed-DPI hosting remains a separate test.
        if (!SetProcessDpiAwarenessContext(new IntPtr(-4))) throw new InvalidOperationException("Cannot establish PMv2");
        Application.SetUnhandledExceptionMode(UnhandledExceptionMode.ThrowException);
        Application.EnableVisualStyles();
        if (args.Length == 1 && args[0] == "--fixture-child")
        {
            using (var child = new Form { Text = "Synthetic workspace client", Bounds = new Rectangle(80, 80, 480, 320) })
            using (var lifetime = new Timer { Interval = 20000 })
            {
                child.Controls.Add(new Label { Text = "Synthetic client — no Minecraft or accounts", Dock = DockStyle.Fill, TextAlign = ContentAlignment.MiddleCenter });
                lifetime.Tick += delegate { child.Close(); };
                lifetime.Start();
                Application.Run(child);
            }
            return 0;
        }
        if (args.Length != 2 || (args[0] != "normal" && args[0] != "partial-failure")) return 2;
        bool injectFailure = args[0] == "partial-failure", attached = false, restored = false, partialObserved = false;
        string errorText = null;
        Process process = null;
        IntPtr window = IntPtr.Zero, originalParent = IntPtr.Zero;
        long originalStyle = 0;
        Rect originalRect = new Rect();
        DateTime started = DateTime.MinValue;
        using (var host = new Form { Text = "Workspace hosting probe", Bounds = new Rectangle(160, 160, 800, 560), ShowInTaskbar = false })
        using (var tick = new Timer { Interval = 100 })
        {
            int stage = 0;
            Stopwatch elapsed = Stopwatch.StartNew();
            host.Shown += delegate
            {
                File.AppendAllText(args[1] + ".trace", "start-child\n");
                process = Process.Start(new ProcessStartInfo(Application.ExecutablePath, "--fixture-child") { UseShellExecute = false });
                started = process.StartTime.ToUniversalTime();
                tick.Start();
            };
            Action restore = delegate
            {
                if (!attached) return;
                CheckOwned(process, started, window);
                Reparent(window, originalParent);
                SetStyle(window, originalStyle);
                Require(GetParent(window) == originalParent, "Parent restoration failed");
                Place(window, originalRect.Left, originalRect.Top, originalRect.Right - originalRect.Left, originalRect.Bottom - originalRect.Top);
                Require(GetWindowLongPtrW(window, Style).ToInt64() == originalStyle, "Style restoration failed");
                Rect rect;
                Require(GetWindowRect(window, out rect) && rect.Left == originalRect.Left && rect.Top == originalRect.Top && rect.Right == originalRect.Right && rect.Bottom == originalRect.Bottom, "Geometry restoration failed");
                restored = true;
                attached = false;
            };
            tick.Tick += delegate
            {
                try
                {
                    if (elapsed.ElapsedMilliseconds > 8000) throw new TimeoutException("Fixture deadline exceeded");
                    if (stage == 0)
                    {
                        process.Refresh();
                        window = process.MainWindowHandle;
                        if (window == IntPtr.Zero) return;
                        File.AppendAllText(args[1] + ".trace", "child-window\n");
                        CheckOwned(process, started, window);
                        Require(AreDpiAwarenessContextsEqual(GetWindowDpiAwarenessContext(window), GetWindowDpiAwarenessContext(host.Handle)), "DPI contexts differ");
                        originalParent = GetParent(window);
                        Require(originalParent == IntPtr.Zero, "Fixture is not an unowned top-level window");
                        originalStyle = GetWindowLongPtrW(window, Style).ToInt64();
                        Require(GetWindowRect(window, out originalRect), "Cannot save geometry");
                        // Mark before mutation so any partially completed operation restores.
                        attached = true;
                        SetStyle(window, (originalStyle & ~Popup) | Child);
                        Reparent(window, host.Handle);
                        Require(GetParent(window) == host.Handle, "Parent readback failed");
                        File.AppendAllText(args[1] + ".trace", "attached\n");
                        Place(window, 0, 0, host.ClientSize.Width, host.ClientSize.Height);
                        Require((GetWindowLongPtrW(window, Style).ToInt64() & Child) != 0, "Child style missing");
                        if (injectFailure) { partialObserved = true; throw new InvalidOperationException("Injected attach failure"); }
                        stage = 1;
                    }
                    else
                    {
                        CheckOwned(process, started, window);
                        restore();
                        File.AppendAllText(args[1] + ".trace", "restored\n");
                        tick.Stop();
                        host.Close();
                    }
                }
                catch (Exception ex)
                {
                    File.AppendAllText(args[1] + ".trace", ex.ToString() + "\n");
                    if (!partialObserved || ex.Message != "Injected attach failure") errorText = ex.Message;
                    try { restore(); } catch (Exception cleanup) { errorText = "Restore failed: " + cleanup.Message; }
                    tick.Stop();
                    host.Close();
                }
            };
            host.FormClosing += delegate(object sender, FormClosingEventArgs e)
            {
                // Refuse normal host destruction if a live child has not been restored.
                try { restore(); } catch (Exception ex) { errorText = ex.Message; e.Cancel = true; }
            };
            Application.Run(host);
        }
        bool closed = false;
        if (process != null)
        {
            if (!process.HasExited) { CheckOwned(process, started, window); process.CloseMainWindow(); }
            closed = process.WaitForExit(4000);
            process.Dispose();
        }
        bool passed = errorText == null && restored && closed && (!injectFailure || partialObserved);
        // Fixed sanitized schema; no HWND, PID, account, title or installation path.
        File.WriteAllText(args[1], "{\"result\":\"" + (passed ? "PASS" : "FAIL") + "\",\"restored\":" + restored.ToString().ToLowerInvariant() + ",\"normalClose\":" + closed.ToString().ToLowerInvariant() + ",\"partialFailureObserved\":" + partialObserved.ToString().ToLowerInvariant() + "}");
        return passed ? 0 : 1;
    }
}
