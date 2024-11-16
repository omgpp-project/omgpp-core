#!/usr/bin/env python
import logging
import sys
import json
from typing import List
import string
from google.protobuf.compiler.plugin_pb2 import (
    CodeGeneratorResponse,
    CodeGeneratorRequest,
)
from google.protobuf.descriptor_pb2 import (
    FileDescriptorProto,
)
from languages.csharp.csharp_gen import *
from utils import *
"""
https://buf.build/docs/reference/descriptors/#options-messages
─ FileDescriptorProto
   │
   ├─ DescriptorProto           // Messages
   │   ├─ FieldDescriptorProto  //   - normal fields and nested extensions
   │   ├─ OneofDescriptorProto
   │   ├─ DescriptorProto       //   - nested messages
   │   │   └─ (...more...)
   │   └─ EnumDescriptorProto   //   - nested enums
   │       └─ EnumValueDescriptorProto
   │
   ├─ EnumDescriptorProto       // Enums
   │   └─ EnumValueDescriptorProto
   │
   ├─ FieldDescriptorProto      // Extensions
   │
   └─ ServiceDescriptorProto    // Services
       └─ MethodDescriptorProto
"""
protoc_dev_input_file="dev_protoc_input.txt"



def debug_descriptors(descriptors:List[FileDescriptorProto]):
    for d in descriptors:
        print('=====================')           
        print(d.name)
        print('Package:' ,d.package)
        print('Options:' ,d.options.csharp_namespace, d.options.java_package)
        print('Dependencies: ',len(d.dependency),d.dependency)        # imported protos
        print('Messages: ',len(d.message_type), [get_csharp_name(m.name) for m in d.message_type])      # declared messages
        print('Services: ',len(d.service), [s.name for s in d.service])           # declared services    
        print("csharp_namespace = ",get_namespace(d))
        # names = [to_csharp_name(m.name) for m in d.message_type]
        # for name in names:
            # print(get_csharp_class_template(name)) 
if __name__ == "__main__":
    # save_protoc_input(protoc_dev_input_file)
    request = None
    buffer = io.StringIO() 
    debug=False
    # read from file for debug purpose
    if debug:
        file1 = open("dev_protoc_input.txt", "rb") 
        request = CodeGeneratorRequest.FromString(file1.read())
        file1.close()
    else:
        request = CodeGeneratorRequest.FromString(sys.stdin.buffer.read())
    response = CodeGeneratorResponse()
    
    files_to_generate = request.file_to_generate
    descriptors = request.source_file_descriptors

    namespace_dict = {}
    for desc in descriptors:
        namespace = get_namespace(desc)
        if namespace not in namespace_dict:
            namespace_dict[namespace] = []

        values = namespace_dict[namespace]
        values.append(desc)
        namespace_dict[namespace] = values
    for namespace in namespace_dict:
        descriptors = namespace_dict[namespace]
        for d in descriptors:
            filename = get_output_filename(d.name)
            extension = "Omgpp.cs"
            csharp_gen_protoc(buffer,namespace,d)
            response.file.append(CodeGeneratorResponse.File(name=f"{filename}.{extension}",content=buffer.getvalue()))
            buffer = io.StringIO()

    sys.stdout.buffer.write(response.SerializeToString())