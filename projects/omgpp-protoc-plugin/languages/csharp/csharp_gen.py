import io
import sys
from typing import List
 
from google.protobuf.descriptor_pb2 import (
    FileDescriptorProto,
    DescriptorProto,
)
def to_upper(string:str):
   return string.replace(string[0],string[0].upper(),1)

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

def get_csharp_name(message_name:str) -> str:
    return "".join(to_upper(part) for part in message_name.split("_"))

def get_message_id_internal(text):
    i = 0
    id = 0
    # just sum up each character;
    # to reflect a position of character in result ID just multiply it by position
    for char in text: 
        id = id + ord(char) * i
        i = i+1
    return id    # only Integer

def get_message_id(message:DescriptorProto,descriptor:FileDescriptorProto):
    csharp_name = get_csharp_name(message.name)
    package = descriptor.package or "EMPTY"
    full_qualified_name = ".".join([package,csharp_name])
    return get_message_id_internal(full_qualified_name)


def with_csharp_namespace_surrounding(buffer: io.StringIO,namespace,callback):
    if namespace is None or len(namespace) == 0:
        callback(buffer)
        return
    buffer.write(f"namespace {namespace}\n")
    buffer.write("{\n")
    callback(buffer)
    buffer.write("}\n")

def process_message(buffer:io.StringIO,message:DescriptorProto,descriptor:FileDescriptorProto):
    csharp_name = get_csharp_name(message.name)
    message_id = get_message_id(message,descriptor)
    buffer.write(f"public sealed partial class {csharp_name} : IOmgppMessage, IOmgppMessage<{csharp_name}> \n")
    buffer.write("{\n")
    buffer.write(f"\tpublic static long MessageId {{get;}} = {message_id};\n")
    buffer.write(f"\tpublic static MessageParser<{csharp_name}> MessageParser => Parser;\n")
    buffer.write("}\n")
    
def process_service(buffer:io.StringIO,service,descriptor:FileDescriptorProto):

    pass
def process_file_descriptor(buffer: io.StringIO,descriptor:FileDescriptorProto):
    for message in descriptor.message_type:
        process_message(buffer,message,descriptor)
    for service in descriptor.service:
        process_service(buffer,service,descriptor)

def process_usings(buffer: io.StringIO):
    buffer.write("using global::OmgppSharpCore.Interfaces;\n")
    buffer.write("using Google.Protobuf;\n")

def csharp_gen_protoc(buffer:io.StringIO,namespace:str,descriptors:FileDescriptorProto):
    process_usings(buffer)
    with_csharp_namespace_surrounding(buffer,namespace,lambda buf: process_file_descriptor(buf,descriptors))