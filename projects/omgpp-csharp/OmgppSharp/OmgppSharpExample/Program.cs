using System.Net;
using awd.awd;
using Google.Protobuf;
using OmgppNative;
using OmgppSharpCore;
using OmgppSharpServer;

namespace OmgppSharpExample
{
    internal class Program
    {
        static void Main(string[] args)
        {
            var server = new Server("127.0.0.1", 55655);
            server.OnConnectionRequest = OnConnectionRequest;
            server.OnConnectionStateChanged += OnConnectionStateChanged;
            server.OnRawMessage+= OnRawMessage;
            var t = new Thread(() =>
            {
                while (true)
                {
                    server.Process();
                }
            });
            t.Start();
            Console.ReadLine();
            MessageHandler handler = new MessageHandler();
            handler.RegisterOnMessage<Message>((message) =>
            {
                Console.WriteLine(message);
            });
            handler.RegisterOnMessage<awd.awd.Void>((@void) =>
            {
                Console.WriteLine(@void);
            });
            handler.RegisterOnMessage<awd.awd.MessageTest>((test) =>
            {
                Console.WriteLine(test);
            });


            var messageData = new Message { Type = 123, Data = ByteString.CopyFrom(1, 2, 3, 4, 5) };
            var testData = new MessageTest { Field1 = 0, StringField = "Some string", BytesField = ByteString.CopyFrom(0, 0, 0, 0, 0, 0) };
            var @void = new awd.awd.Void();
            var memoryStream = new MemoryStream();

            messageData.WriteTo(memoryStream);
            handler.HandleRawMessage(Message.MessageId, memoryStream.ToArray());
            memoryStream.SetLength(0);

            testData.WriteTo(memoryStream);
            handler.HandleRawMessage(MessageTest.MessageId, memoryStream.ToArray());
            memoryStream.SetLength(0);

            @void.WriteTo(memoryStream);
            handler.HandleRawMessage(awd.awd.Void.MessageId, memoryStream.ToArray());
            memoryStream.SetLength(0);
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
            Console.WriteLine($"Connection Request from Id {guid} {address}:{port}");
            return true;
        }
    }
}
