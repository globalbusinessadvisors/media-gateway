/**
 * MCP Server Core
 * Handles tool calls, resource requests, and prompt generation
 */

import { MCPErrorCode, UserContext } from './types/index.js';
import { mcpTools, toolExecutors } from './tools/index.js';
import { mcpResources, getResourceContent } from './resources/index.js';
import { mcpPrompts, getPromptContent } from './prompts/index.js';

/**
 * Main request handler for MCP operations
 */
export async function handleMCPRequest(method: string, params: any): Promise<any> {
  switch (method) {
    case 'initialize':
      return handleInitialize();

    case 'tools/list':
      return handleToolsList();

    case 'tools/call':
      return handleToolCall(params);

    case 'resources/list':
      return handleResourcesList();

    case 'resources/read':
      return handleResourceRead(params);

    case 'prompts/list':
      return handlePromptsList();

    case 'prompts/get':
      return handlePromptGet(params);

    default:
      throw {
        code: MCPErrorCode.METHOD_NOT_FOUND,
        message: `Method not found: ${method}`,
      };
  }
}

/**
 * Initialize MCP server capabilities
 */
function handleInitialize() {
  return {
    protocolVersion: '2024-11-05',
    capabilities: {
      tools: {
        listChanged: false,
      },
      resources: {
        listChanged: false,
      },
      prompts: {
        listChanged: false,
      },
    },
    serverInfo: {
      name: 'media-gateway-mcp',
      version: '1.0.0',
    },
  };
}

/**
 * List available tools
 */
function handleToolsList() {
  return {
    tools: mcpTools,
  };
}

/**
 * Execute a tool
 */
async function handleToolCall(params: {
  name: string;
  arguments: any;
  userContext?: UserContext;
}): Promise<any> {
  const { name, arguments: args, userContext } = params;

  const executor = (toolExecutors as any)[name];
  if (!executor) {
    throw {
      code: MCPErrorCode.TOOL_NOT_FOUND,
      message: `Tool not found: ${name}`,
    };
  }

  try {
    const result = await executor(args, userContext);
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  } catch (error) {
    console.error(`[Server] Tool execution error for ${name}:`, error);
    throw {
      code: MCPErrorCode.TOOL_EXECUTION_ERROR,
      message: error instanceof Error ? error.message : 'Tool execution failed',
      data: error,
    };
  }
}

/**
 * List available resources
 */
function handleResourcesList() {
  return {
    resources: mcpResources,
  };
}

/**
 * Read a resource
 */
async function handleResourceRead(params: { uri: string }): Promise<any> {
  const { uri } = params;

  try {
    const content = await getResourceContent(uri);
    const resource = mcpResources.find((r) => r.uri === uri);

    if (!resource) {
      throw new Error(`Resource not found: ${uri}`);
    }

    return {
      contents: [
        {
          uri,
          mimeType: resource.mimeType || 'application/json',
          text: typeof content === 'string' ? content : JSON.stringify(content, null, 2),
        },
      ],
    };
  } catch (error) {
    console.error(`[Server] Resource read error for ${uri}:`, error);
    throw {
      code: MCPErrorCode.RESOURCE_NOT_FOUND,
      message: error instanceof Error ? error.message : 'Resource not found',
    };
  }
}

/**
 * List available prompts
 */
function handlePromptsList() {
  return {
    prompts: mcpPrompts,
  };
}

/**
 * Get a prompt
 */
function handlePromptGet(params: { name: string; arguments?: Record<string, string> }): any {
  const { name, arguments: args = {} } = params;

  const prompt = mcpPrompts.find((p) => p.name === name);
  if (!prompt) {
    throw {
      code: MCPErrorCode.RESOURCE_NOT_FOUND,
      message: `Prompt not found: ${name}`,
    };
  }

  // Validate required arguments
  const requiredArgs = prompt.arguments?.filter((arg) => arg.required) || [];
  for (const arg of requiredArgs) {
    if (!args[arg.name]) {
      throw {
        code: MCPErrorCode.INVALID_PARAMS,
        message: `Missing required argument: ${arg.name}`,
      };
    }
  }

  try {
    const content = getPromptContent(name, args);
    return {
      description: prompt.description,
      messages: [
        {
          role: 'user',
          content: {
            type: 'text',
            text: content,
          },
        },
      ],
    };
  } catch (error) {
    console.error(`[Server] Prompt generation error for ${name}:`, error);
    throw {
      code: MCPErrorCode.INTERNAL_ERROR,
      message: error instanceof Error ? error.message : 'Prompt generation failed',
    };
  }
}
