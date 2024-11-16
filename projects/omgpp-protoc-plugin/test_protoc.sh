protoc -I ./proto --csharp_out=./generated proto/message.proto proto/services.proto --omgpp_out=./generated --plugin=protoc-gen-omgpp=proto-omgpp-gen.py
