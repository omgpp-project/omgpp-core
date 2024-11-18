using Google.Protobuf;
using OmgppNative;
using OmgppSharpCore;
using OmgppSharpCore.Interfaces;
using System;
using System.Diagnostics;
using System.Net;
using System.Net.Http.Headers;
using System.Runtime.InteropServices;
using System.Text;

namespace OmgppSharpServer
{
    unsafe public class Server : IDisposable
    {
        delegate void ConnectionStateChangedCallbackDelegate(UuidFFI player, EndpointFFI endpoint, ConnectionState state);
        delegate bool ConnectionRequestedCallbackDelegate(UuidFFI player, EndpointFFI endpoint);
        delegate void MessageCallbackDelegate(UuidFFI player, EndpointFFI endpoint, long messageId, byte* data, uint size);

        public Func<Server,Guid, IPAddress, ushort, bool> OnConnectionRequest;
        public event Action<Server,Guid, IPAddress,ushort, ConnectionState> OnConnectionStateChanged;
        public event Action<Server,Guid, IPAddress,ushort, long, byte[]> OnRawMessage;

        private IntPtr _handle;
        private bool _disposed;
        private MessageHandler _messageHandler = new MessageHandler();
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
        public void Process()
        {
            OmgppServerNative.server_process(_handle.ToPointer());
        }

        public void RegisterOnMessage<T>(Action<T> callback) where T : IOmgppMessage<T>, IMessage<T>
        {
            _messageHandler.RegisterOnMessage(callback);
        }
        private void OnMessage(UuidFFI player, EndpointFFI endpoint, long messageId, byte* data, uint size)
        {
            var guid = GuidFromFFI(player);
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            var dataSpan = new Span<byte>(data, (int)size).ToArray();
            OnRawMessage?.Invoke(this,guid,ip,port,messageId,dataSpan);
            _messageHandler.HandleRawMessage(messageId,dataSpan);
        }

        private bool OnConnectionRequested(UuidFFI player, EndpointFFI endpoint)
        {
            if (OnConnectionRequest == null)
                return true;

            var bytes = new Span<byte>(endpoint.ipv6_octets, 16);
            var port = endpoint.port;
            IPAddress address = new IPAddress(bytes);

            return OnConnectionRequest.Invoke(this,new Guid(new Span<byte>(player.bytes, 16)), address, port);
        }


        private void HandleOnConnectionChanged(UuidFFI player, EndpointFFI endpoint, ConnectionState state)
        {
            var guid = GuidFromFFI(player);
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            OnConnectionStateChanged?.Invoke(this, guid,ip,port,state);
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

        private IPAddress IpAddressFromEndpoint(EndpointFFI endpoint)
        {
            var bytes = new Span<byte>(endpoint.ipv6_octets, 16);
            return new IPAddress(bytes);
        }
        private Guid GuidFromFFI(UuidFFI uuid)
        {
            return new Guid(new Span<byte>(uuid.bytes, 16));
        }
    }
}
