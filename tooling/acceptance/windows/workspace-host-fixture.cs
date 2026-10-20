// Synthetic windows only. Never accepts an external PID or HWND.
using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.IO.Pipes;
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
            if (args.Length == 2 && args[0] != "crash-host" && args[0] != "crash-surface") File.WriteAllText(args[1] + ".error", ex.ToString());
            return 1;
        }
    }
    static int Run(string[] args)
    {
        // Match the manager's PMv2 context. Mixed-DPI hosting remains a separate test.
        if (!SetProcessDpiAwarenessContext(new IntPtr(-4))) throw new InvalidOperationException("Cannot establish PMv2");
        Application.SetUnhandledExceptionMode(UnhandledExceptionMode.ThrowException);
        Application.EnableVisualStyles();
        if (args.Length == 2 && args[0] == "crash-recovery") return CrashRecovery(args[1], false);
        if (args.Length == 2 && args[0] == "independent-surface-crash") return CrashRecovery(args[1], true);
        if (args.Length == 2 && args[0] == "protected-manager-crash") return ProtectedManagerCrash(args[1]);
        if (args.Length == 1 && args[0] == "--fixture-child")
        {
            using (var child = new Form { Text = "Synthetic workspace client", Bounds = new Rectangle(80, 80, 480, 320) })
            using (var lifetime = new Timer { Interval = 20000 })
            // A destroyed native HWND can strand the Forms message loop and its timer.
            // This process-only safety exit is restricted to synthetic clients.
            using (var safety = new System.Threading.Timer(delegate { Environment.Exit(3); }, null, 20000, System.Threading.Timeout.Infinite))
            {
                child.Controls.Add(new Label { Text = "Synthetic client — no Minecraft or accounts", Dock = DockStyle.Fill, TextAlign = ContentAlignment.MiddleCenter });
                lifetime.Tick += delegate { child.Close(); };
                lifetime.Start();
                Application.Run(child);
            }
            return 0;
        }
        if (args.Length != 2 || (args[0] != "normal" && args[0] != "partial-failure" && args[0] != "crash-host" && args[0] != "crash-surface")) return 2;
        bool independent = args[0] == "crash-surface";
        bool crashHost = args[0] == "crash-host" || independent;
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
                if (!crashHost) File.AppendAllText(args[1] + ".trace", "start-child\n");
                process = Process.Start(new ProcessStartInfo(Application.ExecutablePath, "--fixture-child") { UseShellExecute = false });
                started = process.StartTime.ToUniversalTime();
                tick.Start();
            };
            Action restore = delegate
            {
                if (!attached) return;
                CheckOwned(process, started, window);
                if (!independent) Reparent(window, originalParent);
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
                        if (!crashHost) File.AppendAllText(args[1] + ".trace", "child-window\n");
                        CheckOwned(process, started, window);
                        Require(AreDpiAwarenessContextsEqual(GetWindowDpiAwarenessContext(window), GetWindowDpiAwarenessContext(host.Handle)), "DPI contexts differ");
                        originalParent = GetParent(window);
                        Require(originalParent == IntPtr.Zero, "Fixture is not an unowned top-level window");
                        originalStyle = GetWindowLongPtrW(window, Style).ToInt64();
                        Require(GetWindowRect(window, out originalRect), "Cannot save geometry");
                        // Mark before mutation so any partially completed operation restores.
                        attached = true;
                        if (independent)
                        {
                            // Proposal only: a native top-level surface aligned with the host.
                            // Never give it a manager parent/owner, and never synthesize input.
                            SetStyle(window, originalStyle & ~0x00C40000L); // Caption and thick frame.
                            Point origin = host.PointToScreen(Point.Empty);
                            Place(window, origin.X, origin.Y, host.ClientSize.Width, host.ClientSize.Height);
                            Require(GetParent(window) == IntPtr.Zero, "Independent surface acquired a parent");
                        }
                        else
                        {
                            SetStyle(window, (originalStyle & ~Popup) | Child);
                            Reparent(window, host.Handle);
                            Require(GetParent(window) == host.Handle, "Parent readback failed");
                            Place(window, 0, 0, host.ClientSize.Width, host.ClientSize.Height);
                            Require((GetWindowLongPtrW(window, Style).ToInt64() & Child) != 0, "Child style missing");
                        }
                        if (!crashHost) File.AppendAllText(args[1] + ".trace", "attached\n");
                        if (crashHost)
                        {
                            // Only the coordinator's freshly spawned fixture uses this pipe.
                            using (var pipe = new NamedPipeClientStream(".", args[1], PipeDirection.Out))
                            {
                                pipe.Connect(2000);
                                using (var writer = new StreamWriter(pipe))
                                {
                                    writer.WriteLine(string.Join("|", new object[] { process.Id, started.Ticks, window.ToInt64(), originalStyle, originalRect.Left, originalRect.Top, originalRect.Right, originalRect.Bottom }));
                                }
                            }
                        }
                        if (injectFailure) { partialObserved = true; throw new InvalidOperationException("Injected attach failure"); }
                        stage = 1;
                    }
                    else
                    {
                        if (crashHost) return; // Coordinator deliberately terminates this synthetic host.
                        CheckOwned(process, started, window);
                        restore();
                        File.AppendAllText(args[1] + ".trace", "restored\n");
                        tick.Stop();
                        host.Close();
                    }
                }
                catch (Exception ex)
                {
                    if (!crashHost) File.AppendAllText(args[1] + ".trace", ex.ToString() + "\n");
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
            bool requested = false;
            if (!process.HasExited) { CheckOwned(process, started, window); requested = process.CloseMainWindow(); }
            closed = requested && process.WaitForExit(4000) && process.ExitCode == 0;
            process.Dispose();
        }
        bool passed = errorText == null && restored && closed && (!injectFailure || partialObserved);
        // Fixed sanitized schema; no HWND, PID, account, title or installation path.
        File.WriteAllText(args[1], "{\"result\":\"" + (passed ? "PASS" : "FAIL") + "\",\"restored\":" + restored.ToString().ToLowerInvariant() + ",\"normalClose\":" + closed.ToString().ToLowerInvariant() + ",\"partialFailureObserved\":" + partialObserved.ToString().ToLowerInvariant() + "}");
        return passed ? 0 : 1;
    }

    static int CrashRecovery(string output, bool independent)
    {
        bool survived = false, restored = false, closed = false;
        Process client = null;
        IntPtr window = IntPtr.Zero;
        DateTime started = DateTime.MinValue;
        string pipeName = "MivubiSyntheticHost-" + Guid.NewGuid().ToString("N");
        using (var pipe = new NamedPipeServerStream(pipeName, PipeDirection.In, 1, PipeTransmissionMode.Byte, PipeOptions.Asynchronous))
        using (var host = Process.Start(new ProcessStartInfo(Application.ExecutablePath, (independent ? "crash-surface " : "crash-host ") + pipeName) { UseShellExecute = false }))
        {
            try
            {
                var connection = pipe.BeginWaitForConnection(null, null);
                using (var ready = connection.AsyncWaitHandle)
                {
                    Require(ready.WaitOne(6000), "Crash host connection timed out");
                    pipe.EndWaitForConnection(connection);
                }
                using (var reader = new StreamReader(pipe))
                {
                    var pending = reader.ReadLineAsync();
                    Require(pending.Wait(2000), "Crash host identity timed out");
                    string[] fields = pending.Result.Split('|');
                    Require(fields.Length == 8, "Invalid synthetic identity");
                    client = Process.GetProcessById(int.Parse(fields[0]));
                    // Retain an OS handle before exit so ExitCode is readable for this IPC-discovered fixture.
                    Require(client.Handle != IntPtr.Zero, "Cannot retain synthetic client handle");
                    started = new DateTime(long.Parse(fields[1]), DateTimeKind.Utc);
                    window = new IntPtr(long.Parse(fields[2]));
                    CheckOwned(client, started, window);
                    Require(string.Equals(client.MainModule.FileName, Application.ExecutablePath, StringComparison.OrdinalIgnoreCase), "Not the synthetic executable");
                    Require(independent ? GetParent(window) == IntPtr.Zero : GetParent(window) != IntPtr.Zero, "Unexpected surface parent before crash");
                    // Kill only this Process.Start result. The synthetic client is separate.
                    host.Kill();
                    Require(host.WaitForExit(3000), "Synthetic host did not exit");
                    survived = !client.HasExited && IsWindow(window);
                    if (survived)
                    {
                        CheckOwned(client, started, window);
                        if (!independent) Reparent(window, IntPtr.Zero);
                        SetStyle(window, long.Parse(fields[3]));
                        int left = int.Parse(fields[4]), top = int.Parse(fields[5]);
                        int right = int.Parse(fields[6]), bottom = int.Parse(fields[7]);
                        Place(window, left, top, right - left, bottom - top);
                        Rect rect;
                        restored = GetParent(window) == IntPtr.Zero && GetWindowLongPtrW(window, Style).ToInt64() == long.Parse(fields[3]) && GetWindowRect(window, out rect) && rect.Left == left && rect.Top == top && rect.Right == right && rect.Bottom == bottom;
                    }
                }
            }
            finally
            {
                if (!host.HasExited) { host.Kill(); host.WaitForExit(3000); }
                if (client != null)
                {
                    bool requested = false;
                    if (!client.HasExited && IsWindow(window)) { CheckOwned(client, started, window); requested = client.CloseMainWindow(); }
                    closed = requested && client.WaitForExit(4000) && client.ExitCode == 0;
                    if (!closed && !client.HasExited)
                    {
                        // A destroyed HWND can leave this synthetic GUI's message loop alive.
                        // This cleanup is forbidden for real games; only this compiled fixture qualifies.
                        Require(!client.HasExited && client.StartTime.ToUniversalTime() == started && string.Equals(client.MainModule.FileName, Application.ExecutablePath, StringComparison.OrdinalIgnoreCase), "Synthetic cleanup identity changed");
                        client.Kill();
                        Require(client.WaitForExit(2000), "Synthetic cleanup timed out");
                    }
                    client.Dispose();
                }
            }
        }
        bool passed = survived && restored && closed;
        File.WriteAllText(output, "{\"result\":\"" + (passed ? "PASS" : "FAIL") + "\",\"survivedHostCrash\":" + survived.ToString().ToLowerInvariant() + ",\"restored\":" + restored.ToString().ToLowerInvariant() + ",\"normalClose\":" + closed.ToString().ToLowerInvariant() + "}");
        return passed ? 0 : 1;
    }

    static int ProtectedManagerCrash(string output)
    {
        bool survived = false, restored = false, closed = false;
        string error = null;
        using (var surface = new Form { Text = "Synthetic presentation owner", Bounds = new Rectangle(160, 160, 800, 560), ShowInTaskbar = false })
        using (var timer = new Timer { Interval = 100 })
        using (var manager = Process.Start(new ProcessStartInfo(Application.ExecutablePath, "--fixture-child") { UseShellExecute = false }))
        using (var client = Process.Start(new ProcessStartInfo(Application.ExecutablePath, "--fixture-child") { UseShellExecute = false }))
        {
            DateTime managerStart = manager.StartTime.ToUniversalTime(), clientStart = client.StartTime.ToUniversalTime();
            IntPtr gameWindow = IntPtr.Zero;
            long gameStyle = 0;
            Rect gameRect = new Rect();
            bool attached = false;
            int stage = 0;
            Stopwatch elapsed = Stopwatch.StartNew();
            Action restore = delegate
            {
                if (!attached) return;
                CheckOwned(client, clientStart, gameWindow);
                Reparent(gameWindow, IntPtr.Zero);
                SetStyle(gameWindow, gameStyle);
                Place(gameWindow, gameRect.Left, gameRect.Top, gameRect.Right - gameRect.Left, gameRect.Bottom - gameRect.Top);
                Rect rect;
                restored = GetParent(gameWindow) == IntPtr.Zero && GetWindowLongPtrW(gameWindow, Style).ToInt64() == gameStyle && GetWindowRect(gameWindow, out rect) && rect.Left == gameRect.Left && rect.Top == gameRect.Top && rect.Right == gameRect.Right && rect.Bottom == gameRect.Bottom;
                Require(restored, "Protected presentation restoration failed");
                attached = false;
            };
            surface.Shown += delegate { timer.Start(); };
            timer.Tick += delegate
            {
                try
                {
                    Require(elapsed.ElapsedMilliseconds < 8000, "Protected fixture deadline exceeded");
                    if (stage == 0)
                    {
                        client.Refresh(); manager.Refresh();
                        gameWindow = client.MainWindowHandle;
                        IntPtr managerWindow = manager.MainWindowHandle;
                        if (gameWindow == IntPtr.Zero || managerWindow == IntPtr.Zero) return;
                        CheckOwned(client, clientStart, gameWindow);
                        CheckOwned(manager, managerStart, managerWindow);
                        Require(AreDpiAwarenessContextsEqual(GetWindowDpiAwarenessContext(gameWindow), GetWindowDpiAwarenessContext(surface.Handle)), "DPI mismatch");
                        Require(GetParent(gameWindow) == IntPtr.Zero && GetParent(managerWindow) == IntPtr.Zero, "Expected fresh synthetic windows");
                        gameStyle = GetWindowLongPtrW(gameWindow, Style).ToInt64();
                        Require(GetWindowRect(gameWindow, out gameRect), "Cannot save synthetic game geometry");
                        attached = true;
                        SetStyle(gameWindow, (gameStyle & ~Popup) | Child);
                        Reparent(gameWindow, surface.Handle);
                        Place(gameWindow, 0, 60, surface.ClientSize.Width, surface.ClientSize.Height - 60);
                        SetStyle(managerWindow, (GetWindowLongPtrW(managerWindow, Style).ToInt64() & ~Popup) | Child);
                        Reparent(managerWindow, surface.Handle);
                        Place(managerWindow, 0, 0, surface.ClientSize.Width, 60);
                        Require(GetParent(gameWindow) == surface.Handle && GetParent(managerWindow) == surface.Handle, "Protected parent readback failed");
                        // Kill only the fake UI. The presentation owner and synthetic game are separate.
                        manager.Kill();
                        stage = 1;
                    }
                    else
                    {
                        if (!manager.HasExited) return;
                        CheckOwned(client, clientStart, gameWindow);
                        survived = GetParent(gameWindow) == surface.Handle;
                        Require(survived, "UI failure destroyed the synthetic game surface");
                        restore();
                        timer.Stop();
                        surface.Close();
                    }
                }
                catch (Exception ex)
                {
                    error = ex.ToString();
                    try { restore(); } catch (Exception recovery) { error += "\n" + recovery; }
                    timer.Stop();
                    surface.Close();
                }
            };
            surface.FormClosing += delegate { restore(); };
            try { Application.Run(surface); }
            finally
            {
                restore();
                if (!manager.HasExited) { manager.CloseMainWindow(); if (!manager.WaitForExit(2000)) { manager.Kill(); manager.WaitForExit(2000); } }
                bool requested = false;
                if (!client.HasExited && IsWindow(gameWindow)) { CheckOwned(client, clientStart, gameWindow); requested = client.CloseMainWindow(); }
                closed = requested && client.WaitForExit(4000) && client.ExitCode == 0;
            }
        }
        if (error != null) File.WriteAllText(output + ".error", error);
        bool passed = error == null && survived && restored && closed;
        File.WriteAllText(output, "{\"result\":\"" + (passed ? "PASS" : "FAIL") + "\",\"survivedManagerCrash\":" + survived.ToString().ToLowerInvariant() + ",\"restored\":" + restored.ToString().ToLowerInvariant() + ",\"normalClose\":" + closed.ToString().ToLowerInvariant() + "}");
        return passed ? 0 : 1;
    }
}
