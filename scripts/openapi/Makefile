# Common variables
OPENAPI_DIR := ../../../openapi
OPENAPI_BACKEND_AGENT_DIR := $(OPENAPI_DIR)/configs/backend-agent
OPENAPI_AGENT_SDK_DIR := $(OPENAPI_DIR)/configs/agent-sdk

# Agent API variables
OPENAPI_CLIENT_FILE := $(OPENAPI_BACKEND_AGENT_DIR)/openapi.gen.yaml
OPENAPI_SERVER_FILE := $(OPENAPI_AGENT_SDK_DIR)/openapi.gen.yaml
CLIENT_GEN := gen/client
SERVER_GEN := gen/server

gen: clean
	make gen-client
	make gen-server

gen-client: clean-client
	openapi-generator-cli generate \
	-i ${OPENAPI_CLIENT_FILE} \
	-g rust \
	-o ${CLIENT_GEN} \
	--additional-properties=packageName=openapi-client

gen-server: clean-server
	openapi-generator-cli generate \
	-i ${OPENAPI_SERVER_FILE} \
	-g rust \
	-o ${SERVER_GEN} \
	--additional-properties=packageName=openapi-server

clean: clean-server clean-client

clean-server:
	rm -rf ${SERVER_GEN}

clean-client:
	rm -rf ${CLIENT_GEN}