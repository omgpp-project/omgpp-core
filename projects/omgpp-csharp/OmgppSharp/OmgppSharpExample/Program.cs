using awd.awd;
using Google.Protobuf;
using OmgppSharpCore;

namespace OmgppSharpExample
{
    internal class Program
    {
        static void Main(string[] args)
        {
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
    }
}
