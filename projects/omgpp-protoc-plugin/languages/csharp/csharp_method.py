
from typing import List, Tuple

class CSharpMethod:
    def __init__(self,id:int, name:str,return_type:str,input_args:List[Tuple[str,str]],has_output:bool,has_input_message:bool) -> None:
        self.id = id
        self.name =name
        self.return_type = return_type
        self.input_args = input_args
        self.has_output = has_output
        self.has_input_message = has_input_message