using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Text;
using System.Threading.Tasks;

namespace OmgppSharpServer
{
    public interface IServerRpcHandler
    {
        public delegate void ServerRpcHandlerDelegate(Server server, Guid clientGuid, IPAddress ip, ushort port, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData);

        public void HandleRpc(Server server, Guid clientGuid, IPAddress ip, ushort port, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData);

    }
}
