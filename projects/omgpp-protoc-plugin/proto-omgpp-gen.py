#!/usr/bin/python
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
        print('Messages: ',len(d.message_type), [to_camel_case(m.name) for m in d.message_type])      # declared messages
        print('Services: ',len(d.service), [s.name for s in d.service])           # declared services    
        print("csharp_namespace = ",get_namespace(d))
        # names = [to_csharp_name(m.name) for m in d.message_type]
        # for name in names:
            # print(get_csharp_class_template(name)) 
if __name__ == "__main__":
    #save_protoc_input(protoc_dev_input_file)
    request = None
    debug=False
    # read from file for debug purpose
    if debug:
        file1 = open(protoc_dev_input_file, "rb") 
        request = CodeGeneratorRequest.FromString(file1.read())
        file1.close()
    else:
        request = CodeGeneratorRequest.FromString(sys.stdin.buffer.read())
    
    files_to_generate = request.file_to_generate
    descriptors = request.source_file_descriptors
    context = DescriptorContext(descriptors)

    response = csharp_gen_omgpp(context)

    # if not debug:
    sys.stdout.buffer.write(response.SerializeToString())