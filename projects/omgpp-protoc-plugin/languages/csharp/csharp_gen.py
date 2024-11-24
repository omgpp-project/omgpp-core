import io
import sys
from typing import List
 
from google.protobuf.descriptor_pb2 import (
    FileDescriptorProto,
    ServiceDescriptorProto,
    DescriptorProto,
)
from google.protobuf.compiler.plugin_pb2 import (
    CodeGeneratorResponse,
)
from descriptor_context import DescriptorContext
from languages.csharp.csharp_method import CSharpMethod
from languages.csharp.csharp_rpc_templates import get_rpc_client_handler, get_rpc_server_handler
from utils import get_message_id, get_method_id, get_output_filename, to_camel_case, to_upper


def save_protoc_input(filename):
    f = open(filename,'wb')
    data = sys.stdin.buffer.read()
    f.write(data)
    f.close()

def get_namespace(d:FileDescriptorProto) -> str:
    csharp_namespace = None
    if len(d.package) > 0:
        csharp_namespace = ".".join([to_upper(part) for part in d.package.split(".")])
    if len(d.options.csharp_namespace) > 0:
        csharp_namespace = d.options.csharp_namespace
    return csharp_namespace





def with_csharp_namespace_surrounding(buffer: io.StringIO,namespace,callback):
    if namespace is None or len(namespace) == 0:
        callback(buffer)
        return
    buffer.write(f"namespace {namespace}\n")
    buffer.write("{\n")
    callback(buffer)
    buffer.write("}\n")

def process_message(buffer:io.StringIO,message:DescriptorProto,descriptor:FileDescriptorProto):
    csharp_name = to_camel_case(message.name)
    message_id = get_message_id(message,descriptor)
    buffer.write(f"public sealed partial class {csharp_name} : IOmgppMessage, IOmgppMessage<{csharp_name}> \n")
    buffer.write("{\n")
    buffer.write(f"\tpublic static long MessageId {{get;}} = {message_id};\n")
    buffer.write(f"\tpublic static MessageParser<{csharp_name}> MessageParser => Parser;\n")
    buffer.write("}\n")
    
def process_service(buffer:io.StringIO,service:ServiceDescriptorProto,descriptor:FileDescriptorProto,context:DescriptorContext,is_server:bool):
    csharp_methods = []
    for method in service.method:
        id = get_method_id(service,method,descriptor)
        is_input_empty = method.input_type == ".google.protobuf.Empty"
        is_out_empty = method.output_type == ".google.protobuf.Empty"
        (input_arg_message,input_arg_file_descriptor)  = context.get_message_descriptor(method.input_type)
        (out_arg_message,out_arg_file_descriptor)  = context.get_message_descriptor(method.output_type)
        if input_arg_message is None and not is_input_empty:
            raise Exception(f"Cannot find {method.input_type} message; Make sure you provided all .proto files to process")
        if out_arg_message is None and not is_out_empty:
            raise Exception(f"Cannot find {method.output_type} message; Make sure you provided all .proto files to process")

        if is_input_empty:
            full_qualified_csharp_input = "void"
        else:        
            full_qualified_csharp_input = f"global::{get_namespace(input_arg_file_descriptor)}.{to_camel_case(input_arg_message.name)}"
        
        if is_out_empty:
            full_qualified_csharp_output = "void"
        else:
            full_qualified_csharp_output = f"global::{get_namespace(out_arg_file_descriptor)}.{to_camel_case(out_arg_message.name)}"

        if is_input_empty:
            csharp_methods.append(CSharpMethod(id,method.name,full_qualified_csharp_output,[],not is_out_empty,False))
        else:
            csharp_methods.append(CSharpMethod(id,method.name,full_qualified_csharp_output,[(full_qualified_csharp_input,"message")],not is_out_empty,True))
    

    if is_server:
        gen_rpc_server_interface(buffer, service.name, csharp_methods)
        gen_rpc_server_handler(buffer,service.name,csharp_methods)
    else:
        gen_rpc_client_interface(buffer, service.name, csharp_methods)
        gen_rpc_client_handler(buffer,service.name,csharp_methods)

def gen_rpc_server_handler(buffer, service_name, methods: List[CSharpMethod]):
    buffer.write(get_rpc_server_handler(service_name,methods))
def gen_rpc_client_handler(buffer,service_name,methods: List[CSharpMethod]):
    buffer.write(get_rpc_client_handler(service_name,methods))

def gen_rpc_server_interface(buffer, service_name, methods: List[CSharpMethod]):
    default_args = [("System.Guid","clientGuid"), ("System.Net.IPAddress","ip"), ("ushort","port")]
    interface_name = "I"+service_name+"Server"
    buffer.write(f"public interface {interface_name}\n")
    buffer.write("{\n")
    for m in methods:
        all_input_args = default_args + m.input_args
        input_args = ",".join(map(lambda arg: f"{arg[0]} {arg[1]}",all_input_args))
        buffer.write(f"\t{m.return_type} {m.name}({input_args});\n")
        pass
    buffer.write("}\n")

def gen_rpc_client_interface(buffer, service_name, methods: List[CSharpMethod]):
    interface_name = "I"+service_name+"Client"
    buffer.write(f"public interface {interface_name}\n")
    buffer.write("{\n")
    for m in methods:
        all_input_args = m.input_args + [("bool","isReliable")]
        input_args = ",".join(map(lambda arg: f"{arg[0]} {arg[1]}",all_input_args))
        if m.has_output:
            buffer.write(f"\tTask<{m.return_type}> {m.name}({input_args});\n")
        else:
            buffer.write(f"\t{m.return_type} {m.name}({input_args});\n")
        pass
    buffer.write("}\n")
    
def process_messages_in_file_descriptor(buffer: io.StringIO,descriptor:FileDescriptorProto,context:DescriptorContext):
    for message in descriptor.message_type:
        process_message(buffer,message,descriptor)

def process_services_in_file_descriptor(buffer: io.StringIO,descriptor:FileDescriptorProto,context:DescriptorContext,is_server:bool):
    for service in descriptor.service:
        process_service(buffer,service,descriptor,context,is_server)

def process_messages_usings(buffer: io.StringIO):
    buffer.write("using global::OmgppSharpCore.Interfaces;\n")
    buffer.write("using Google.Protobuf;\n")

def process_service_server_usings(buffer: io.StringIO):
    buffer.write("using global::OmgppSharpCore.Interfaces;\n")
    buffer.write("using Google.Protobuf;\n")
    buffer.write("using OmgppSharpServer;\n")
    buffer.write("using static OmgppSharpServer.IServerRpcHandler;\n")


def process_client_server_using(buffer: io.StringIO):
    buffer.write("using global::OmgppSharpCore.Interfaces;\n")
    buffer.write("using Google.Protobuf;\n")
    buffer.write("using OmgppSharpClient;\n")
def process_header(buffer: io.StringIO):
    buffer.write("/** <auto-generated>\n")
    buffer.write("* Do not edit this file manually  \n")
    buffer.write("* This file was generated by proto-omgpp-gen.py 0.1.0 \n")
    buffer.write("* Any changes will be discarded after regeneration\n")
    buffer.write("* </auto-generated>\n")
    buffer.write("*/ \n")

def csharp_gen_proto_messages(buffer:io.StringIO,namespace:str,descriptors:FileDescriptorProto, context:DescriptorContext):
    process_header(buffer)
    process_messages_usings(buffer)
    with_csharp_namespace_surrounding(buffer,namespace,lambda buf: process_messages_in_file_descriptor(buf,descriptors,context))

def csharp_gen_proto_services_server(buffer:io.StringIO,namespace:str,descriptors:FileDescriptorProto, context:DescriptorContext):
    process_header(buffer)
    process_service_server_usings(buffer)
    with_csharp_namespace_surrounding(buffer,namespace,lambda buf: process_services_in_file_descriptor(buf,descriptors,context,True))

def csharp_gen_proto_services_client(buffer:io.StringIO,namespace:str,descriptors:FileDescriptorProto, context:DescriptorContext):
    process_header(buffer)
    process_client_server_using(buffer)
    with_csharp_namespace_surrounding(buffer,namespace,lambda buf: process_services_in_file_descriptor(buf,descriptors,context,False))

def csharp_gen_omgpp(descriptor_context:DescriptorContext) -> CodeGeneratorResponse:
    namespace_dict = {}
    response = CodeGeneratorResponse()
    buffer = io.StringIO() 
    for desc in descriptor_context.descriptors:
        namespace = get_namespace(desc)
        if namespace not in namespace_dict:
            namespace_dict[namespace] = []

        values = namespace_dict[namespace]
        values.append(desc)
        namespace_dict[namespace] = values

    for namespace in namespace_dict:
        namespace_descriptors = namespace_dict[namespace]
        for descriptor in namespace_descriptors:
            filename = get_output_filename(descriptor.name)
            extension = "Omgpp.cs"
            csharp_gen_proto_messages(buffer,namespace,descriptor,descriptor_context)
            response.file.append(CodeGeneratorResponse.File(name=f"{filename}.{extension}",content=buffer.getvalue()))
            buffer = io.StringIO()
            if descriptor.service is not None and len(descriptor.service) > 0:
                csharp_gen_proto_services_server(buffer,namespace,descriptor,descriptor_context)
                response.file.append(CodeGeneratorResponse.File(name=f"{filename}.Service.Server.{extension}",content=buffer.getvalue())) 
                buffer = io.StringIO()
                csharp_gen_proto_services_client(buffer,namespace,descriptor,descriptor_context)
                response.file.append(CodeGeneratorResponse.File(name=f"{filename}.Service.Client.{extension}",content=buffer.getvalue())) 
                buffer = io.StringIO()
            pass
            
    return response