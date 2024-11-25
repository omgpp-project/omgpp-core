
from typing import List
from languages.csharp.csharp_method import CSharpMethod

def get_rpc_client_handler(service_name:str,service_methods:List[CSharpMethod]):

    methods = ""
    for m in service_methods:
        input_arg_name = m.input_args[0][0] if m.has_input_message else ""
        out_param = f"Task<{m.return_type}>" if m.has_output else m.return_type
        all_input_args = m.input_args + [("bool","isReliable")]
        input_args = ",".join(map(lambda arg: f"{arg[0]} {arg[1]}",all_input_args))

        method = f"public {out_param} {m.name}({input_args})\n"
        method += "{\n"
        if not m.has_output:
            if m.has_input_message:
                method += f""" 
          var size = message.CalculateSize();
          var bytes = ArrayPool<byte>.Shared.Rent(size);
          message.WriteTo(bytes);
          client.CallRpc({m.id}, 0, {input_arg_name}.MessageId, bytes, isReliable);
          ArrayPool<byte>.Shared.Return(bytes);
"""
            else:
                method += f"client.CallRpc({m.id}, 0, 0, null, isReliable);"
        else:
            if  m.has_input_message:
                method += f""" 
          var size = message.CalculateSize();
          var bytes = ArrayPool<byte>.Shared.Rent(size);
          message.WriteTo(bytes);
"""
            else:
               method += f"byte[] bytes= Array.Empty<byte>();"

            method += f"""
          var taskCompletionSource = new TaskCompletionSource<{m.return_type}>();
          var reqId = Interlocked.Increment(ref this.reqId);
          var cancellationToken = new CancellationTokenSource();
          cancellationToken.CancelAfter(1000);

          if(rpcResponseHandlers.ContainsKey(reqId))
          {{
              ArrayPool<byte>.Shared.Return(bytes);
              throw new Exception("Internal error; Request Id already registered");
          }}

          var tokenRegisterHandler = cancellationToken.Token.Register(() =>
          {{
              rpcResponseHandlers.Remove(reqId);
              taskCompletionSource.TrySetResult(default);
          }});
          rpcResponseHandlers[reqId] = (client, ip, port, isReliable, methodId, requestId, argType, argData) =>
          {{
              tokenRegisterHandler.Unregister();
              if (argType != {m.return_type}.MessageId || argData == null || cancellationToken.Token.IsCancellationRequested)
              {{
                  taskCompletionSource.TrySetResult(default);
              }}
              else
              {{
                  var msg = {m.return_type}.Parser.ParseFrom(argData);
                  taskCompletionSource.TrySetResult(msg);
              }}
              rpcResponseHandlers.Remove(reqId);
          }};
          client.CallRpc({m.id}, reqId, {input_arg_name}.MessageId,bytes,isReliable);
          ArrayPool<byte>.Shared.Return(bytes);
          return taskCompletionSource.Task;
"""
        method += "}\n"

        methods += method

    return f"""
    public class {service_name}ClientHandler : I{service_name}Client, IDisposable
    {{
      OmgppSharpClient.Client client;
      Dictionary<long, OmgppSharpClient.IClientRpcHandler.ClientRpcHandlerDelegate> rpcHandlers = new Dictionary<long, OmgppSharpClient.IClientRpcHandler.ClientRpcHandlerDelegate>();
      Dictionary<ulong, OmgppSharpClient.IClientRpcHandler.ClientRpcHandlerDelegate> rpcResponseHandlers = new Dictionary<ulong, OmgppSharpClient.IClientRpcHandler.ClientRpcHandlerDelegate>();
      ulong reqId = 0;
      public {service_name}ClientHandler(OmgppSharpClient.Client client)
      {{
          this.client = client;
          this.client.OnRpcCall += Client_OnRpcCall;
      }}

      private void Client_OnRpcCall(Client client, System.Net.IPAddress remoteIp, ushort remotePort, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData)
      {{
          if(rpcResponseHandlers.TryGetValue(requestId, out var handler))
              handler.Invoke(client,remoteIp,remotePort, isReliable, methodId, requestId, argType, argData);
      }}
      public void Dispose()
      {{
          this.client.OnRpcCall -= Client_OnRpcCall;
      }}

      void RegisterRpc(long id, OmgppSharpClient.IClientRpcHandler.ClientRpcHandlerDelegate handlerAction)
      {{
          rpcHandlers[id] = handlerAction;
      }}

      {methods}
   }}
    """
    pass


def get_rpc_server_handler(service_name,service_methods:List[CSharpMethod]):
    handle_methods = ""
    for m in service_methods:
        method = f"private void Handle{m.name}(Server server, Guid clientGuid, IPAddress ip, ushort port, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData)"
        method += "{\n"
        if not m.has_output:
            if m.has_input_message:
                method += f"if (argType != {m.input_args[0][0]}.MessageId) return;\n"
                method += f"argData = argData?? Array.Empty<byte>();\n"
                method += f"service.{m.name}(clientGuid, ip, port, {m.input_args[0][0]}.Parser.ParseFrom(argData));\n"
            else:
                method += f"service.{m.name}(clientGuid, ip, port);\n"
        else:
            if  m.has_input_message:
                method += f"if (argType != {m.input_args[0][0]}.MessageId) return;\n"
                method += f"argData = argData?? Array.Empty<byte>();\n"
                method += f"var result = service.{m.name}(clientGuid, ip, port, {m.input_args[0][0]}.Parser.ParseFrom(argData));\n"
            else:
                method += f"var result = service.{m.name}(clientGuid, ip, port);\n"
            
            method += f"""var size = result.CalculateSize();
                var data = ArrayPool<byte>.Shared.Rent(size);
                result.WriteTo(data);
                server.CallRpc(clientGuid, methodId, requestId, {m.input_args[0][0]}.MessageId, data, true);
                ArrayPool<byte>.Shared.Return(data);\n
"""

        method += "}\n"

        handle_methods += method


    register_handle_methods = ""
    for m in service_methods:
        register_handle_methods += f"RegisterRpc({m.id}, Handle{m.name});\n"

    return f"""
    public class {service_name}ServerHandler : global::OmgppSharpServer.IServerRpcHandler
    {{
        I{service_name}Server service;
        Dictionary<long,ServerRpcHandlerDelegate> rpcHandlers = new Dictionary<long, ServerRpcHandlerDelegate>();

        public {service_name}ServerHandler(I{service_name}Server service)
        {{
            this.service = service;
            {register_handle_methods}
        }}
        {handle_methods}
 
        public void HandleRpc(Server server, Guid clientGuid, IPAddress ip, ushort port, bool isReliable, long methodId, ulong requestId, long argType, byte[]? argData)
        {{
            if (rpcHandlers.TryGetValue(methodId, out var handler))
                handler.Invoke(server, clientGuid, ip, port,isReliable,methodId,requestId, argType, argData);
        }}
        void RegisterRpc(long id,ServerRpcHandlerDelegate handlerAction)
        {{
            rpcHandlers[id] = handlerAction;
        }}
    }}
"""