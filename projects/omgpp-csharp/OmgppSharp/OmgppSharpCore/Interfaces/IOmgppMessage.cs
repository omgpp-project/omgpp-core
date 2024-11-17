﻿using Google.Protobuf;

namespace OmgppSharpCore.Interfaces
{
    public interface IOmgppMessage
    {
        static abstract int MessageId { get; }
    }
    public interface IOmgppMessage<T> : IOmgppMessage where T : IMessage<T>
    {
        static abstract MessageParser<T> MessageParser { get; }
    }
}