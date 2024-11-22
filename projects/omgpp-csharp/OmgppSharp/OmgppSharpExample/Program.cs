using System.Net;
using System.Text;
using awd.awd;
using Google.Protobuf;
using OmgppNative;
using OmgppSharpCore;
using OmgppSharpServer;
namespace OmgppSharpExample
{
    internal class Program
    {
        static Guid LastClient = Guid.Empty;
        unsafe static void Main(string[] args)
        {

            var server = new Server("127.0.0.1", 55655);
            server.OnConnectionRequest = OnConnectionRequest;
            server.OnConnectionStateChanged += OnConnectionStateChanged;
            server.OnRawMessage+= OnRawMessage;
            server.OnRpcCall += Server_OnRpcCall;
            var t = new Thread(() =>
            {
                while (true)
                {
                    server.Process();
                }
            });

            t.Start();
            bool end = false;   
            while (!end)
            {
                var str = Console.ReadLine();
                end = str == "end";
                if (end || str == null)
                    continue;
                var msgParams = str.Split(" ");
                bool isBroadcast = msgParams[0] == "b";
                if (isBroadcast)
                {
                    server.Broadcast(888, Encoding.UTF8.GetBytes(str));
                }else
                {
                    server.Send(LastClient, 888, Encoding.UTF8.GetBytes(str));
                }
            }
        }

        private static void Server_OnRpcCall(Server server, Guid client, IPAddress ip, ushort port, bool reliable, long methodId, ulong requestId, long argType, byte[]? data)
        {
            Console.WriteLine($"{client} {ip}:{port} reliable={reliable} Method={methodId} Req={requestId} Arg={argType} Data={data?.Length}");
            server.CallRpcBroadcast(methodId, requestId, argType, data, reliable);  
        }

        private static void OnConnectionStateChanged(Server server,Guid guid, IPAddress address, ushort port, ConnectionState state)
        {
            Console.WriteLine($"ConnectionState changed Id {guid} {address}:{port} {state}");
        }

        private static void OnRawMessage(Server server,Guid guid, IPAddress address, ushort port, long messageId, byte[] data)
        {
            Console.WriteLine($"Message from Id {guid} {address}:{port} {messageId} length {data.Length}");
        }

        private static bool OnConnectionRequest(Server server,Guid guid, IPAddress address, ushort port)
        {
            LastClient = guid;
            Console.WriteLine($"Connection Request from Id {guid} {address}:{port}");
            return true;
        }
    }
}
