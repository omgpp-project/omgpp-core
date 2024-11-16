def get_output_filename(proto_file_name:str) -> str:
    without_extension = proto_file_name.split('.')
    if len(without_extension) > 1:
        without_extension = "".join(without_extension[:-1])
    else:
        without_extension = ""

    return "".join(part.capitalize() for part in without_extension.split("_"))