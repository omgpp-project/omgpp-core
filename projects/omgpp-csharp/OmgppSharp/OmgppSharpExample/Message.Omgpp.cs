using global::OmgppSharpCore.Interfaces;
using Google.Protobuf;
namespace awd.awd
{
public sealed partial class Void : IOmgppMessage, IOmgppMessage<Void> 
{
	public static long MessageId {get;} = 18589;
	public static MessageParser<Void> MessageParser => Parser;
}
public sealed partial class Message : IOmgppMessage, IOmgppMessage<Message> 
{
	public static long MessageId {get;} = 25065;
	public static MessageParser<Message> MessageParser => Parser;
}
public sealed partial class MessageTest : IOmgppMessage, IOmgppMessage<MessageTest> 
{
	public static long MessageId {get;} = 35312;
	public static MessageParser<MessageTest> MessageParser => Parser;
}
}
