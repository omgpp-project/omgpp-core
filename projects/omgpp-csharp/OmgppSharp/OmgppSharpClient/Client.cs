using OmgppNative;
using System.Net;
using System.Runtime.InteropServices;
using System.Text;
using static System.Runtime.InteropServices.JavaScript.JSType;

namespace OmgppSharpClient
{
    unsafe public class Client : IDisposable
    {
        private delegate void ConnectionStateChangedNativeDelegate(EndpointFFI endpoint, ConnectionState state);
        private delegate void RawMessageNativeDelegate(EndpointFFI endpoint, long messageId, byte* data, uint size);
        private delegate void RpcCallNativeDelegate(EndpointFFI endpoint, bool reliable, long methodId, ulong requestId, long argType, byte* argData, uint size);


        public delegate void RawMessageDelegate(Client client, IPAddress remoteIp, ushort remotePort, long messageId, byte[] messageData);
        public delegate void ConnectionStateChangedDelegate(Client client, IPAddress remoteIp, ushort remotePort, ConnectionState state);
        public delegate void RpcCallDelegate(Client client, IPAddress remoteIp, ushort remotePort, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData);


        public event RawMessageDelegate OnRawMessage;
        public event ConnectionStateChangedDelegate OnConnectionStateChanged;
        public event RpcCallDelegate OnRpcCall;

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
            var ptr = Marshal.GetFunctionPointerForDelegate(new ConnectionStateChangedNativeDelegate(HandleOnConnectionChateChangedNative));
            OmgppClientNative.client_register_on_connection_state_change(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<EndpointFFI, ConnectionState, void>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new RawMessageNativeDelegate(HandleOnMessageNative));
            OmgppClientNative.client_register_on_message(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<EndpointFFI, long, byte*, nuint, void>)ptr);

            ptr = Marshal.GetFunctionPointerForDelegate(new RpcCallNativeDelegate(HandleOnRpcCallNative));
            OmgppClientNative.client_register_on_rpc(_handle.ToPointer(), (delegate* unmanaged[Cdecl]<EndpointFFI, bool, long, ulong, long, byte*, nuint, void>)ptr);
        }



        public void Connect()
        {
            OmgppClientNative.client_connect(_handle.ToPointer());
        }
        public void Disconnect()
        {
            OmgppClientNative.client_disconnect(_handle.ToPointer());
        }

        public void Send(long messageId, Span<byte> data)
        {
            fixed (byte* dataPtr = data)
            {
                OmgppClientNative.client_send(_handle.ToPointer(), messageId, dataPtr,0,(nuint)data.Length);
            }
        }
        public void SendReliable(long messageId, Span<byte> data)
        {
            fixed (byte* dataPtr = data)
            {
                OmgppClientNative.client_send_reliable(_handle.ToPointer(), messageId, dataPtr,0, (nuint)data.Length);
            }
        }
        public void CallRpc(long methodId, ulong requestId, long argType, Span<byte> argData, bool reliable)
        {
            fixed (byte* argDataPtr = argData)
            {
                OmgppClientNative.client_call_rpc(_handle.ToPointer(), reliable, methodId, requestId, argType, argDataPtr, 0,(nuint)argData.Length);
            }
        }
        public void Send(long messageId, byte[] data,int offset,int length)
        {
            var span = new Span<byte>(data, offset, length);
            Send(messageId, span);

        }
        public void SendReliable(long messageId, byte[] data, int offset, int length)
        {
            var span = new Span<byte>(data, offset, length);
            SendReliable(messageId, span);
        }
        public void CallRpc(long methodId, ulong requestId, long argType, byte[]? argData,int offset,int length, bool reliable)
        {
            var span = new Span<byte>(argData, offset, length);
            CallRpc(methodId,requestId,argType,span,reliable);
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
            OnConnectionStateChanged?.Invoke(this, ip, port, State);
        }
        private void HandleOnMessageNative(EndpointFFI endpoint, long messageId, byte* data, uint size)
        {
            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            var msgBytes = new Span<byte>(data, (int)size).ToArray();
            OnRawMessage?.Invoke(this, ip, port, messageId, msgBytes);
        }
        private void HandleOnRpcCallNative(EndpointFFI endpoint, bool reliable, long methodId, ulong requestId, long argType, byte* argData, uint argDataSize)
        {
            if (OnRpcCall == null)
                return;

            var ip = IpAddressFromEndpoint(endpoint);
            var port = endpoint.port;
            var data = argDataSize == 0 ? null : new Span<byte>(argData, (int)argDataSize).ToArray();
            OnRpcCall?.Invoke(this, ip, port, reliable, methodId, requestId, argType, data);
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
