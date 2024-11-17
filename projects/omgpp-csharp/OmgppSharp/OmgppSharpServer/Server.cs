using OmgppNative;
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

namespace OmgppSharpServer
{
    unsafe public class Server : IDisposable
    {
        delegate void ConnectionStateChangedCallbackDelegate(UuidFFI player, EndpointFFI endpoint, ConnectionState state);
        delegate bool ConnectionRequestedCallbackDelegate(UuidFFI player, EndpointFFI endpoint);
        delegate void MessageCallbackDelegate(UuidFFI player, EndpointFFI endpoint, long messageId, byte* data, uint size);


        private IntPtr _handle;
        private bool _disposed;

        public Server(string ip, ushort port)
        {
            fixed (byte* cstr = Encoding.UTF8.GetBytes(ip))
            {
                _handle = new IntPtr(OmgppServerNative.server_create(cstr, port));
                if (_handle == IntPtr.Zero)
                    throw new Exception("Cannot create a server");
            }

            var ptr = Marshal.GetFunctionPointerForDelegate(new ConnectionRequestedCallbackDelegate(OnConnectionRequested));
            OmgppServerNative.server_register_on_connect_requested(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<UuidFFI, EndpointFFI, bool>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new ConnectionStateChangedCallbackDelegate(HandleOnConnectionChanged));
            OmgppServerNative.server_register_on_connection_state_change(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<UuidFFI, EndpointFFI, ConnectionState, void>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new MessageCallbackDelegate(OnMessage));
            OmgppServerNative.server_register_on_message(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<UuidFFI, EndpointFFI, long, byte*, nuint, void>)ptr);
        }

        private void OnMessage(UuidFFI player, EndpointFFI endpoint, long messageId, byte* data, uint size)
        {
            var guid = player.bytes;
            Span<byte> buffer = new Span<byte>(guid, 16);

            var bytes = new Span<byte>(data, (int)size).ToArray();
            Console.WriteLine($"{new Guid(buffer)} {messageId} {bytes}");
        }

        private bool OnConnectionRequested(UuidFFI player, EndpointFFI endpoint)
        {
            return true;
        }

        public void Process()
        {
            OmgppServerNative.server_process(_handle.ToPointer());
        }

        private void HandleOnConnectionChanged(UuidFFI player, EndpointFFI endpoint, ConnectionState state)
        {
            var guid = player.bytes;
            Span<byte> buffer = new Span<byte>(guid, 16);
            Console.WriteLine($"{new Guid(buffer)} {state}");
        }

        public void Dispose()
        {
            if (!_disposed)
            {
                OmgppServerNative.server_destroy(_handle.ToPointer());
                _handle = IntPtr.Zero;
                _disposed = true;
            }
        }
        private void EnsureAlive()
        {
            if (_handle == IntPtr.Zero)
                throw new Exception("Server handler not alive");
        }
    }
}
