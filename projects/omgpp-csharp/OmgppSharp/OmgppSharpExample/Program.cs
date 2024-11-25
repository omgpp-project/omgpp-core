using System.Net;
using System.Text;
using awd.awd;
using Google.Protobuf;
using OmgppNative;
using OmgppSharpClient;
using OmgppSharpCore;
using OmgppSharpServer;
using Sample.Messages;
namespace OmgppSharpExample
{
    internal class Program
    {
        public static int PORT = 55655;

        class RpcServer : IGameCommandsServer
        {
            public void MoveDown(Guid clientGuid, IPAddress ip, ushort port, Message message)
            {
                Console.WriteLine($"Client {clientGuid} MoveDown");
            }

            public awd.awd.Void MoveLeft(Guid clientGuid, IPAddress ip, ushort port, awd.awd.Void message)
            {
                Console.WriteLine($"Client {clientGuid} MoveLeft");
                return new awd.awd.Void();
            }

            public MessageTest MoveRight(Guid clientGuid, IPAddress ip, ushort port, Message message)
            {
                Console.WriteLine($"Client {clientGuid} MoveRight");
                return new MessageTest();
            }
            public void MoveUp(Guid clientGuid, IPAddress ip, ushort port)
            {
                Console.WriteLine($"Client {clientGuid} MoveUp");
            }
        }

        unsafe static void Main(string[] args)
        {
            if (args.Length == 0)
                throw new Exception("Not enought args");
            var cmd = int.Parse(args[0]);
            if (cmd == 1)
            {
                StartServer();
            }
            else
            {
                StartClient();
            }
        }



        private static void StartServer()
        {
            Console.WriteLine($"Hello i am Server on {PORT}");
            var server = new Server("127.0.0.1", (ushort)PORT);
            server.OnConnectionRequest = OnConnectionRequest;
            server.OnConnectionStateChanged += OnConnectionStateChanged;
            server.OnRawMessage += OnRawMessage;
            server.RegisterRpcHandler(new GameCommandsServerHandler(new RpcServer()));

            var t = new Thread(() =>
            {
                while (true)
                {
                    server.Process();
                    Thread.Sleep(8);
                }
            });
            t.Start();
            Console.ReadLine();
        }
        private static async void StartClient()
        {
            Console.WriteLine($"Hello i am Client on {PORT}");
            var client = new Client("127.0.0.1", (ushort)PORT);
            client.OnConnectionStateChanged += (client, ip, port, state) => { System.Console.WriteLine(state); };
            client.OnRawMessage += (client, ip, port, msgType, msgData) => { System.Console.WriteLine(msgType); };
            client.Connect();
            var rpcHandler = new GameCommandsClientHandler(client);
            var t = new Thread(() =>
            {
                while (true)
                {
                    client.Process();
                    Thread.Sleep(8);
                }
            });
            t.Start();

            while (true)
            {
                var cmd = Console.ReadLine().Trim();
                if (cmd == "q")
                    return;
                switch (cmd)
                {
                    case "u":
                        Console.WriteLine($"RPC request Up");
                        rpcHandler.MoveUp(true);
                        break;
                    case "l":
                        Console.WriteLine($"RPC request Left");
                        var leftResponce = await rpcHandler.MoveLeft(new awd.awd.Void(), true);
                        Console.WriteLine($"RPC Left response {leftResponce}");
                        break;
                    case "r":
                        Console.WriteLine($"RPC request Right");
                        var rightResponse = await rpcHandler.MoveRight(new Message(), true);
                        Console.WriteLine($"RPC Right response {rightResponse}");
                        break;
                    case "d":
                        Console.WriteLine($"RPC request Down");
                        rpcHandler.MoveDown(new Message(), true);
                        break;
                    default: break;
                }
            }
        }

        private static void OnConnectionStateChanged(Server server, Guid guid, IPAddress address, ushort port, ConnectionState state)
        {
            Console.WriteLine($"ConnectionState changed Id {guid} {address}:{port} {state}");
        }

        private static void OnRawMessage(Server server, Guid guid, IPAddress address, ushort port, long messageId, byte[] data)
        {
            Console.WriteLine($"Message from Id {guid} {address}:{port} {messageId} length {data.Length}");
        }

        private static bool OnConnectionRequest(Server server, Guid guid, IPAddress address, ushort port)
        {
            Console.WriteLine($"Connection Request from Id {guid} {address}:{port}");
            return true;
        }
    }
}
