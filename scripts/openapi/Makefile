# Common variables
OPENAPI_DIR := ../../../openapi
OPENAPI_AGENT_DIR := $(OPENAPI_DIR)/configs/agent

# Agent API variables
OPENAPI_FILE := $(OPENAPI_AGENT_DIR)/openapi.gen.yaml
GEN := gen

gen: clean-agent-golang
	openapi-generator-cli generate \
	-i ${OPENAPI_FILE} \
	-g rust \
	-o ${GEN}

clean-agent-golang:
	rm -rf ${GEN}