# Common variables
OPENAPI_DIR := ../../../openapi
OPENAPI_AGENT_DIR := $(OPENAPI_DIR)/configs/agent
OPENAPI_FRONTEND_DIR := $(OPENAPI_DIR)/configs/frontend

# Agent API variables
OPENAPI_SERVER_FILE := $(OPENAPI_AGENT_DIR)/openapi.gen.yaml
OPENAPI_CLIENT_FILE := $(OPENAPI_FRONTEND_DIR)/openapi.gen.yaml
SERVER_GEN := gen/server
CLIENT_GEN := gen/client

gen-server: clean-agent-golang
	openapi-generator-cli generate \
	-i ${OPENAPI_SERVER_FILE} \
	-g rust \
	-o ${SERVER_GEN}

gen-client: clean-agent-golang
	openapi-generator-cli generate \
	-i ${OPENAPI_CLIENT_FILE} \
	-g rust \
	-o ${CLIENT_GEN}

clean-agent:
	rm -rf ${SERVER_GEN}
	rm -rf ${CLIENT_GEN}