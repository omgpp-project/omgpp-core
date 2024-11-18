using OmgppNative;
using System.Net;
using System.Runtime.InteropServices;
using System.Text;

namespace OmgppSharpClient
{
    unsafe public class Client : IDisposable
    {
        delegate void ConnectionStateChangedCallbackDelegate(EndpointFFI endpoint, ConnectionState state);
        delegate void MessageCallbackDelegate(EndpointFFI endpoint, long messageId, byte* data, uint size);

        public event Action<Client,IPAddress,ushort, long, byte[]> OnRawMessage;
        public event Action<Client,IPAddress,ushort, ConnectionState> OnConnectionStateChanged;

        public ConnectionState State { get; private set; } = ConnectionState.None;

        private IntPtr _handle;
        private bool _disposed;

        public Client(string remoteIp, ushort port)
        {
            fixed (byte* cstr = Encoding.UTF8.GetBytes(remoteIp))
            {
                _handle = new IntPtr(OmgppClientNative.client_create(cstr, port));
                if (_handle == IntPtr.Zero)
                    throw new Exception("Cannot create a client");
            }
            var ptr = Marshal.GetFunctionPointerForDelegate(new ConnectionStateChangedCallbackDelegate(HandleOnConnectionChateChangedNative));
            OmgppClientNative.client_register_on_connection_state_change(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<EndpointFFI, ConnectionState, void>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new MessageCallbackDelegate(HandleOnMessageNative));
            OmgppClientNative.client_register_on_message(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<EndpointFFI, long, byte*, nuint, void>)ptr);
        }

        public void Connect()
        {
            OmgppClientNative.client_connect(_handle.ToPointer());
        }
        public void Disconnect()
        {
            OmgppClientNative.client_disconnect(_handle.ToPointer());
        }

        public void Send(long messageId, byte[] data)
        {
            fixed (byte* dataPtr = data)
            {
                OmgppClientNative.client_send(_handle.ToPointer(), messageId, dataPtr, (nuint)data.Length);
            }
        }
        public void SendReliable(long messageId, byte[] data)
        {
            fixed (byte* dataPtr = data)
            {
                OmgppClientNative.client_send_reliable(_handle.ToPointer(), messageId, dataPtr, (nuint)data.Length);
            }
        }
        public void Process()
        {
            OmgppClientNative.client_process(_handle.ToPointer());
        }
        private void HandleOnConnectionChateChangedNative(EndpointFFI endpoint, ConnectionState state)
        {
            State = state;
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            OnConnectionStateChanged?.Invoke(this,ip, port, State);
        }
        private void HandleOnMessageNative(EndpointFFI endpoint, long messageId, byte* data, uint size)
        {
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            var msgBytes =new Span<byte>(data, (int)size).ToArray();
            OnRawMessage?.Invoke(this, ip, port, messageId, msgBytes);
        }


        public void Dispose()
        {
            if (_disposed)
            {
                _disposed = true;
                OmgppClientNative.client_destroy(_handle.ToPointer());
                _handle = IntPtr.Zero;
            }
        }
        private IPAddress IpAddressFromEndpoint(EndpointFFI endpoint)
        {
            var bytes = new Span<byte>(endpoint.ipv6_octets, 16);
            return new IPAddress(bytes);
        }
    }
}
