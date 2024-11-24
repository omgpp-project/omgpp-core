using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Text;
using System.Threading.Tasks;

namespace OmgppSharpClient
{
    public interface IClientRpcHandler
    {
        public delegate void ClientRpcHandlerDelegate(Client client, IPAddress ip, ushort port, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData);
        public void HandleRpc(Client client, IPAddress ip, ushort port, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData);
    }
}
