protos:
	protoc --js_out=import_style=commonjs,binary:./client/libs/ ./proto/bot/bot.proto --plugin=protoc-gen-grpc=`which grpc_tools_node_protoc_plugin`