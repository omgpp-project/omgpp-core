from typing import List, Tuple
from google.protobuf.descriptor_pb2 import (
    FileDescriptorProto,
    DescriptorProto,
)

class DescriptorContext:
    def __init__(self, file_descriptor_array:List[FileDescriptorProto]) -> None:
        self.descriptor_map = dict(map(lambda d: (d.name,d),file_descriptor_array))

        messages_map_keys = []
        for file in file_descriptor_array:
            for message in file.message_type:
                if file.package is None or file.package == "":
                    full_message_name = f".{message.name}"
                else:
                    full_message_name = f".{file.package}.{message.name}"
                messages_map_keys.append((full_message_name,(message,file)))
        self.messages_map = dict(messages_map_keys)

    @property 
    def descriptors(self) -> List[FileDescriptorProto]:
        return self.descriptor_map.values()
    
    def get_message_descriptor(self,full_quialified_message_name:str) -> Tuple[DescriptorProto,FileDescriptorProto]:
       if full_quialified_message_name in self.messages_map:
           return self.messages_map[full_quialified_message_name]
       return (None,None)
    
    def __str__(self) -> str:
        return self.descriptor_map.keys().__str__()