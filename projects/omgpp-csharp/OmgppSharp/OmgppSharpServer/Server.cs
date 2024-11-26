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
using static System.Runtime.InteropServices.JavaScript.JSType;

namespace OmgppSharpServer
{
    unsafe public class Server : IDisposable
    {
        delegate void ConnectionStateChangedNativeDelegate(UuidFFI client, EndpointFFI endpoint, ConnectionState state);
        delegate bool ConnectionRequestedNativeDelegate(UuidFFI client, EndpointFFI endpoint);
        delegate void RawMessageNativeDelegate(UuidFFI client, EndpointFFI endpoint, long messageId, byte* data, uint size);
        delegate void RpcCallNativeDelegate(UuidFFI client, EndpointFFI endpoint, bool reliable, long methodId, ulong requestId, long argType, byte* argData, uint size);

        public delegate bool ConnectionRequestDelegate(Server server, Guid clientGuid, IPAddress ip, ushort port);
        public delegate void ConnectionStateChangedDelegate(Server server, Guid clientGuid, IPAddress ip, ushort port, ConnectionState state);
        public delegate void RawMessageDelegate(Server server, Guid clientGuid, IPAddress ip, ushort port, long messageId, byte[] messageData);
        public delegate void RpcCallDelegate(Server server, Guid clientGuid, IPAddress ip, ushort port, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData);

        public ConnectionRequestDelegate OnConnectionRequest;
        public event ConnectionStateChangedDelegate OnConnectionStateChanged;
        public event RawMessageDelegate OnRawMessage;
        public event RpcCallDelegate OnRpcCall;

        private IntPtr _handle;
        private bool _disposed;
        private MessageHandler _messageHandler = new MessageHandler();
        private List<IServerRpcHandler> _rpcHandlers = new List<IServerRpcHandler>();
        public Server(string ip, ushort port)
        {
            fixed (byte* cstr = Encoding.UTF8.GetBytes(ip))
            {
                _handle = new IntPtr(OmgppServerNative.server_create(cstr, port));
                if (_handle == IntPtr.Zero)
                    throw new Exception("Cannot create a server");
            }

            var ptr = Marshal.GetFunctionPointerForDelegate(new ConnectionRequestedNativeDelegate(OnConnectionRequested));
            OmgppServerNative.server_register_on_connect_requested(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<UuidFFI, EndpointFFI, bool>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new ConnectionStateChangedNativeDelegate(HandleOnConnectionChanged));
            OmgppServerNative.server_register_on_connection_state_change(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<UuidFFI, EndpointFFI, ConnectionState, void>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new RawMessageNativeDelegate(OnMessageNative));
            OmgppServerNative.server_register_on_message(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<UuidFFI, EndpointFFI, long, byte*, nuint, void>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new RpcCallNativeDelegate(OnRcpCallNative));
            OmgppServerNative.server_register_on_rpc(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<UuidFFI, EndpointFFI, bool, long, ulong, long, byte*, nuint, void>)ptr);
        }



        public void Process()
        {
            OmgppServerNative.server_process(_handle.ToPointer());
        }

        public void RegisterOnMessage<T>(Action<T> callback) where T : IOmgppMessage<T>, IMessage<T>
        {
            _messageHandler.RegisterOnMessage(callback);
        }
        public void RegisterRpcHandler(IServerRpcHandler handler)
        {
            if (handler == null)
                return;
            _rpcHandlers.Add(handler);
        }

        public void Send(Guid client, long messageId, Span<byte> data)
        {
            fixed (byte* dataPtr = data)
            {
                var uuidFFi = FfiFromGuid(client);
                OmgppServerNative.server_send(_handle.ToPointer(), &uuidFFi, messageId, dataPtr, 0, (nuint)data.Length);
            }
        }

        public void SendReliable(Guid client, long messageId, Span<byte> data)
        {
            fixed (byte* dataPtr = data)
            {
                var uuidFFi = FfiFromGuid(client);
                OmgppServerNative.server_send_reliable(_handle.ToPointer(), &uuidFFi, messageId, dataPtr,0, (nuint)data.Length);
            }
        }
        public void Broadcast(long messageId, Span<byte> data)
        {
            fixed (byte* dataPtr = data)
            {
                OmgppServerNative.server_broadcast(_handle.ToPointer(), messageId, dataPtr, 0, (nuint)data.Length);
            }
        }
        public void BroadcastReliable(long messageId, Span<byte> data)
        {
            fixed (byte* dataPtr = data)
            {
                OmgppServerNative.server_broadcast_reliable(_handle.ToPointer(), messageId, dataPtr, 0, (nuint)data.Length);
            }
        }
        public void CallRpc(Guid client, long methodId, ulong requestId, long argType, Span<byte> argData, bool reliable)
        {
            fixed (byte* argDataPtr = argData)
            {
                var uuidFFi = FfiFromGuid(client);
                OmgppServerNative.server_call_rpc(_handle.ToPointer(), &uuidFFi, reliable, methodId, requestId, argType, argDataPtr, 0, (nuint)argData.Length);
            }
        }

        public void CallRpcBroadcast(long methodId, ulong requestId, long argType, Span<byte> argData, bool reliable)
        {
            fixed (byte* argDataPtr = argData)
            {
                OmgppServerNative.server_call_rpc_broadcast(_handle.ToPointer(), reliable, methodId, requestId, argType, argDataPtr, 0, (nuint)argData.Length);
            }
        }

        public void Send(Guid client, long messageId, byte[] data, int offset, int length)
        {
            var span = new Span<byte>(data, offset, length);
            Send(client, messageId, span);  
        }

        public void SendReliable(Guid client, long messageId, byte[] data, int offset, int length)
        {
            var span = new Span<byte>(data, offset, length);
            SendReliable(client, messageId, span);
        }
        public void Broadcast(long messageId, byte[] data, int offset, int length)
        {
            var span = new Span<byte>(data, offset, length);
            Broadcast(messageId, span);

        }
        public void BroadcastReliable(long messageId, byte[] data,int offset,int length)
        {
            var span = new Span<byte>(data, offset, length);
            BroadcastReliable(messageId, span);

        }
        public void CallRpc(Guid client, long methodId, ulong requestId, long argType, byte[]? argData, int offset, int length, bool reliable)
        {
            var span = new Span<byte>(argData, offset, length);
            CallRpc(client, methodId, requestId, argType, span, reliable);
        }

        public void CallRpcBroadcast(long methodId, ulong requestId, long argType, byte[]? argData, int offset, int length,bool reliable)
        {
            var span = new Span<byte>(argData,offset, length);
            CallRpcBroadcast(methodId,requestId, argType, span, reliable);
        }
        private void OnMessageNative(UuidFFI client, EndpointFFI endpoint, long messageId, byte* data, uint size)
        {
            var guid = GuidFromFFI(client);
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            var dataSpan = new Span<byte>(data, (int)size).ToArray();
            OnRawMessage?.Invoke(this, guid, ip, port, messageId, dataSpan);
            _messageHandler.HandleRawMessage(messageId, dataSpan);
        }
        private void OnRcpCallNative(UuidFFI client, EndpointFFI endpoint, bool reliable, long methodId, ulong requestId, long argType, byte* argData, uint argDataSize)
        {
            var guid = GuidFromFFI(client);
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            var data = argDataSize == 0 ? null : new Span<byte>(argData, (int)argDataSize).ToArray();
            OnRpcCall?.Invoke(this, guid, ip, port, reliable, methodId, requestId, argType, data);
            foreach (var handler in _rpcHandlers)
            {
                handler?.HandleRpc(this, guid, ip, port, reliable, methodId, requestId, argType, data);
            }
        }

        private bool OnConnectionRequested(UuidFFI client, EndpointFFI endpoint)
        {
            if (OnConnectionRequest == null)
                return true;

            var bytes = new Span<byte>(endpoint.ipv6_octets, 16);
            var port = endpoint.port;
            IPAddress address = new IPAddress(bytes);

            return OnConnectionRequest.Invoke(this, new Guid(new Span<byte>(client.bytes, 16)), address, port);
        }


        private void HandleOnConnectionChanged(UuidFFI client, EndpointFFI endpoint, ConnectionState state)
        {
            var guid = GuidFromFFI(client);
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            OnConnectionStateChanged?.Invoke(this, guid, ip, port, state);
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

        private static UuidFFI FfiFromGuid(Guid client)
        {
            UuidFFI uuidFFi = new UuidFFI();
            var span = new Span<byte>(uuidFFi.bytes, 16);
            client.TryWriteBytes(span);
            return uuidFFi;
        }
    }
}
