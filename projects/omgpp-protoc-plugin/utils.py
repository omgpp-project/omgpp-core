import os
from pathlib import Path

from google.protobuf.descriptor_pb2 import (
    FileDescriptorProto,
    ServiceDescriptorProto,
    MethodDescriptorProto,
    DescriptorProto,
)
def get_output_filename(proto_file_name:str) -> str:
    file = os.path.basename(proto_file_name)
    without_extension = file.split(".")
    if len(without_extension) > 1:
        without_extension = "".join(without_extension[:-1])
    else:
        without_extension = ""

    return "".join(part.capitalize() for part in without_extension.split("_"))

def to_camel_case(message_name:str) -> str:
    return "".join(to_upper(part) for part in message_name.split("_"))
def to_upper(string:str):
   return string.replace(string[0],string[0].upper(),1)

def get_id_from_string(text):
    i = 0
    id = 0
    # just sum up each character;
    # to reflect a position of character in result ID just multiply it by position
    for char in text: 
        id = id + ord(char) * i
        i = i+1
    return id    # only Integer

def get_message_id(message:DescriptorProto,descriptor:FileDescriptorProto):
    csharp_name = to_camel_case(message.name)
    package = descriptor.package or "EMPTY"
    full_qualified_name = ".".join([package,csharp_name])
    return get_id_from_string(full_qualified_name)


def get_method_id(service:ServiceDescriptorProto,method:MethodDescriptorProto, file_descriptor:FileDescriptorProto):
    package = file_descriptor.package or ""
    full_name = ".".join([package,service.name,method.name,method.input_type,method.output_type])
    return get_id_from_string(full_name)
