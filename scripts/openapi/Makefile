# Common variables
OPENAPI_DIR := ../../../openapi
OPENAPI_AGENT_DIR := $(OPENAPI_DIR)/configs/agent
OPENAPI_FRONTEND_DIR := $(OPENAPI_DIR)/all/frontend

# Agent API variables
OPENAPI_SERVER_FILE := $(OPENAPI_AGENT_DIR)/openapi.gen.yaml
OPENAPI_CLIENT_FILE := $(OPENAPI_FRONTEND_DIR)/openapi.gen.yaml
SERVER_GEN := gen/server
CLIENT_GEN := gen/client

gen: clean
	make gen-server
	make gen-client

gen-server: clean-server
	openapi-generator-cli generate \
	-i ${OPENAPI_SERVER_FILE} \
	-g rust \
	-o ${SERVER_GEN} \
	--additional-properties=packageName=openapi-server

gen-client: clean-client
	openapi-generator-cli generate \
	-i ${OPENAPI_CLIENT_FILE} \
	-g rust \
	-o ${CLIENT_GEN} \
	--additional-properties=packageName=openapi-client

clean: clean-server clean-client

clean-server:
	rm -rf ${SERVER_GEN}

clean-client:
	rm -rf ${CLIENT_GEN}