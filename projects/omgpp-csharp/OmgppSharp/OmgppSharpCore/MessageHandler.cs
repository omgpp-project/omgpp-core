using Google.Protobuf;
using OmgppSharpCore.Interfaces;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace OmgppSharpCore
{
    public class MessageHandler
    {
        private Dictionary<long, Action<byte[]>> _handlers = new Dictionary<long, Action<byte[]>>();
        public MessageHandler()
        {

        }

        public void RegisterOnMessage<T>(Action<T> callback) where T : IOmgppMessage<T>, IMessage<T>
        {
            if (callback == null)
                return;
            long id = T.MessageId;
            _handlers[id] = (data) =>
            {
                var result = T.MessageParser.ParseFrom(data);
                callback.Invoke(result);
            };
        }

        public void HandleRawMessage(long id, byte[] data)
        {
            if (_handlers.TryGetValue(id, out var handler))
            {
                handler?.Invoke(data);
            }
        }
    }
}
