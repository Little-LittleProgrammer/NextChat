export const OWNER = "ChatGPTNextWeb";
export const REPO = "ChatGPT-Next-Web";
export const REPO_URL = `https://github.com/${OWNER}/${REPO}`;
export const PLUGINS_REPO_URL = `https://github.com/${OWNER}/NextChat-Awesome-Plugins`;
export const ISSUE_URL = `https://github.com/${OWNER}/${REPO}/issues`;
export const UPDATE_URL = `${REPO_URL}#keep-updated`;
export const RELEASE_URL = `${REPO_URL}/releases`;
export const FETCH_COMMIT_URL = `https://api.github.com/repos/${OWNER}/${REPO}/commits?per_page=1`;
export const FETCH_TAG_URL = `https://api.github.com/repos/${OWNER}/${REPO}/tags?per_page=1`;
export const RUNTIME_CONFIG_DOM = "danger-runtime-config";

export const STABILITY_BASE_URL = "https://api.stability.ai";

export const OPENAI_BASE_URL = "https://api.openai.com";
export const ANTHROPIC_BASE_URL = "https://api.anthropic.com";

export const GEMINI_BASE_URL = "https://generativelanguage.googleapis.com/";

export const BAIDU_BASE_URL = "https://aip.baidubce.com";
export const BAIDU_OATUH_URL = `${BAIDU_BASE_URL}/oauth/2.0/token`;

export const BYTEDANCE_BASE_URL = "https://ark.cn-beijing.volces.com";

export const ALIBABA_BASE_URL = "https://dashscope.aliyuncs.com/api/";

export const MINIMAX_BASE_URL = "https://api.minimaxi.com";

export const TENCENT_BASE_URL = "https://hunyuan.tencentcloudapi.com";

export const MOONSHOT_BASE_URL = "https://api.moonshot.ai";
export const IFLYTEK_BASE_URL = "https://spark-api-open.xf-yun.com";

export const DEEPSEEK_BASE_URL = "https://api.deepseek.com";

export const XAI_BASE_URL = "https://api.x.ai";

export const CHATGLM_BASE_URL = "https://open.bigmodel.cn";

export const SILICONFLOW_BASE_URL = "https://api.siliconflow.cn";

export const AI302_BASE_URL = "https://api.302.ai";

export const CACHE_URL_PREFIX = "/api/cache";
export const UPLOAD_URL = `${CACHE_URL_PREFIX}/upload`;

export enum Path {
  Home = "/",
  Chat = "/chat",
  Settings = "/settings",
  NewChat = "/new-chat",
  Masks = "/masks",
  Plugins = "/plugins",
  Auth = "/auth",
  Sd = "/sd",
  SdNew = "/sd-new",
  Artifacts = "/artifacts",
  SearchChat = "/search-chat",
  McpMarket = "/mcp-market",
}

export enum ApiPath {
  Cors = "",
  Azure = "/api/azure",
  OpenAI = "/api/openai",
  Anthropic = "/api/anthropic",
  Google = "/api/google",
  Baidu = "/api/baidu",
  ByteDance = "/api/bytedance",
  Alibaba = "/api/alibaba",
  MiniMax = "/api/minimax",
  Tencent = "/api/tencent",
  Moonshot = "/api/moonshot",
  Iflytek = "/api/iflytek",
  Stability = "/api/stability",
  Artifacts = "/api/artifacts",
  XAI = "/api/xai",
  ChatGLM = "/api/chatglm",
  DeepSeek = "/api/deepseek",
  SiliconFlow = "/api/siliconflow",
  "302.AI" = "/api/302ai",
}

export enum SlotID {
  AppBody = "app-body",
  CustomModel = "custom-model",
}

export enum FileName {
  Masks = "masks.json",
  Prompts = "prompts.json",
}

export enum StoreKey {
  Chat = "chat-next-web-store",
  Plugin = "chat-next-web-plugin",
  Access = "access-control",
  Config = "app-config",
  Mask = "mask-store",
  Prompt = "prompt-store",
  Update = "chat-update",
  Sync = "sync",
  SdList = "sd-list",
  Mcp = "mcp-store",
}

export const DEFAULT_SIDEBAR_WIDTH = 300;
export const MAX_SIDEBAR_WIDTH = 500;
export const MIN_SIDEBAR_WIDTH = 230;
export const NARROW_SIDEBAR_WIDTH = 100;

export const ACCESS_CODE_PREFIX = "nk-";

export const LAST_INPUT_KEY = "last-input";
export const UNFINISHED_INPUT = (id: string) => "unfinished-input-" + id;

export const STORAGE_KEY = "chatgpt-next-web";

export const REQUEST_TIMEOUT_MS = 60000 * 10;
export const REQUEST_TIMEOUT_MS_FOR_THINKING = REQUEST_TIMEOUT_MS * 5;

export const EXPORT_MESSAGE_CLASS_NAME = "export-markdown";

export enum ServiceProvider {
  OpenAI = "OpenAI",
  Azure = "Azure",
  Google = "Google",
  Anthropic = "Anthropic",
  Baidu = "Baidu",
  ByteDance = "ByteDance",
  Alibaba = "Alibaba",
  Tencent = "Tencent",
  Moonshot = "Moonshot",
  Stability = "Stability",
  Iflytek = "Iflytek",
  XAI = "XAI",
  ChatGLM = "ChatGLM",
  DeepSeek = "DeepSeek",
  SiliconFlow = "SiliconFlow",
  "302.AI" = "302.AI",
  MiniMax = "MiniMax",
}

// Google API safety settings, see https://ai.google.dev/gemini-api/docs/safety-settings
// BLOCK_NONE will not block any content, and BLOCK_ONLY_HIGH will block only high-risk content.
export enum GoogleSafetySettingsThreshold {
  BLOCK_NONE = "BLOCK_NONE",
  BLOCK_ONLY_HIGH = "BLOCK_ONLY_HIGH",
  BLOCK_MEDIUM_AND_ABOVE = "BLOCK_MEDIUM_AND_ABOVE",
  BLOCK_LOW_AND_ABOVE = "BLOCK_LOW_AND_ABOVE",
}

export enum ModelProvider {
  Stability = "Stability",
  GPT = "GPT",
  GeminiPro = "GeminiPro",
  Claude = "Claude",
  Ernie = "Ernie",
  Doubao = "Doubao",
  Qwen = "Qwen",
  Hunyuan = "Hunyuan",
  Moonshot = "Moonshot",
  Iflytek = "Iflytek",
  XAI = "XAI",
  ChatGLM = "ChatGLM",
  DeepSeek = "DeepSeek",
  SiliconFlow = "SiliconFlow",
  "302.AI" = "302.AI",
  MiniMax = "MiniMax",
}

export const Stability = {
  GeneratePath: "v2beta/stable-image/generate",
  ExampleEndpoint: "https://api.stability.ai",
};

export const Anthropic = {
  ChatPath: "v1/messages",
  ChatPath1: "v1/complete",
  ExampleEndpoint: "https://api.anthropic.com",
  Vision: "2023-06-01",
};

export const OpenaiPath = {
  ChatPath: "v1/chat/completions",
  SpeechPath: "v1/audio/speech",
  ImagePath: "v1/images/generations",
  UsagePath: "dashboard/billing/usage",
  SubsPath: "dashboard/billing/subscription",
  ListModelPath: "v1/models",
};

export const Azure = {
  ChatPath: (deployName: string, apiVersion: string) =>
    `deployments/${deployName}/chat/completions?api-version=${apiVersion}`,
  // https://<your_resource_name>.openai.azure.com/openai/deployments/<your_deployment_name>/images/generations?api-version=<api_version>
  ImagePath: (deployName: string, apiVersion: string) =>
    `deployments/${deployName}/images/generations?api-version=${apiVersion}`,
  ExampleEndpoint: "https://{resource-url}/openai",
};

export const Google = {
  ExampleEndpoint: "https://generativelanguage.googleapis.com/",
  ChatPath: (modelName: string) =>
    `v1beta/models/${modelName}:streamGenerateContent`,
};

export const Baidu = {
  ExampleEndpoint: BAIDU_BASE_URL,
  ChatPath: (modelName: string) => {
    let endpoint = modelName;
    if (modelName === "ernie-4.0-8k") {
      endpoint = "completions_pro";
    }
    if (modelName === "ernie-4.0-8k-preview-0518") {
      endpoint = "completions_adv_pro";
    }
    if (modelName === "ernie-3.5-8k") {
      endpoint = "completions";
    }
    if (modelName === "ernie-speed-8k") {
      endpoint = "ernie_speed";
    }
    return `rpc/2.0/ai_custom/v1/wenxinworkshop/chat/${endpoint}`;
  },
};

export const ByteDance = {
  ExampleEndpoint: "https://ark.cn-beijing.volces.com/api/",
  ChatPath: "api/v3/chat/completions",
};

export const Alibaba = {
  ExampleEndpoint: ALIBABA_BASE_URL,
  ChatPath: (modelName: string) => {
    if (
      modelName.includes("vl") ||
      modelName.includes("omni") ||
      modelName.includes("image") ||
      modelName.includes("qvq")
    ) {
      return "v1/services/aigc/multimodal-generation/generation";
    }
    return `v1/services/aigc/text-generation/generation`;
  },
  SpeechPath: "v1/services/aigc/multimodal-generation/generation",
};

export const MiniMax = {
  ExampleEndpoint: MINIMAX_BASE_URL,
  ChatPath: "v1/chat/completions",
};

export const Tencent = {
  ExampleEndpoint: TENCENT_BASE_URL,
};

export const Moonshot = {
  ExampleEndpoint: MOONSHOT_BASE_URL,
  ChatPath: "v1/chat/completions",
};

export const Iflytek = {
  ExampleEndpoint: IFLYTEK_BASE_URL,
  ChatPath: "v1/chat/completions",
};

export const DeepSeek = {
  ExampleEndpoint: DEEPSEEK_BASE_URL,
  ChatPath: "chat/completions",
};

export const XAI = {
  ExampleEndpoint: XAI_BASE_URL,
  ChatPath: "v1/chat/completions",
};

export const ChatGLM = {
  ExampleEndpoint: CHATGLM_BASE_URL,
  ChatPath: "api/paas/v4/chat/completions",
  ImagePath: "api/paas/v4/images/generations",
  VideoPath: "api/paas/v4/videos/generations",
};

export const SiliconFlow = {
  ExampleEndpoint: SILICONFLOW_BASE_URL,
  ChatPath: "v1/chat/completions",
  ListModelPath: "v1/models?&sub_type=chat",
};

export const AI302 = {
  ExampleEndpoint: AI302_BASE_URL,
  ChatPath: "v1/chat/completions",
  EmbeddingsPath: "jina/v1/embeddings",
  ListModelPath: "v1/models?llm=1",
};

export const DEFAULT_INPUT_TEMPLATE = `{{input}}`; // input / time / model / lang
// export const DEFAULT_SYSTEM_TEMPLATE = `
// You are ChatGPT, a large language model trained by {{ServiceProvider}}.
// Knowledge cutoff: {{cutoff}}
// Current model: {{model}}
// Current time: {{time}}
// Latex inline: $x^2$
// Latex block: $$e=mc^2$$
// `;
export const DEFAULT_SYSTEM_TEMPLATE = `
You are ChatGPT, a large language model trained by {{ServiceProvider}}.
Knowledge cutoff: {{cutoff}}
Current model: {{model}}
Current time: {{time}}
Latex inline: \\(x^2\\) 
Latex block: $$e=mc^2$$
`;

export const MCP_TOOLS_TEMPLATE = `
[clientId]
{{ clientId }}
[tools]
{{ tools }}
`;

export const MCP_SYSTEM_TEMPLATE = `
你是一名可以使用系统工具的 AI 助手，你的职责是结合自然语言理解和工具操作来帮助用户完成任务。

1. 可用工具（AVAILABLE TOOLS）:
{{ MCP_TOOLS }}

2. 何时使用工具（WHEN TO USE TOOLS）:
   - 当工具可以帮助回答用户问题时，**务必使用工具**
   - 不要只描述你「可以」做什么——要**立刻实际调用工具执行操作**
   - 如果不确定是否要使用某个工具，**不要使用工具**
   - 典型的工具使用触发场景包括但不限于：
     * 与文件或目录相关的问题
     * 需要检查、列出或操作系统资源的请求
     * 任何可以通过现有工具得到更准确或更可靠答案的查询

3. 如何使用工具（HOW TO USE TOOLS）:
   A. 调用格式（Tool Call Format）:
      - 使用 markdown 代码块，格式为：\`\`\`json:mcp:{clientId}\`\`\`
      - 必须包含以下字段：
        * method: "tools/call"（只支持该方法）
        * params:
          - name: 必须是一个可用 primitive 的名称
          - arguments: 该 primitive 所需的全部参数

   B. 响应格式（Response Format）:
      - 工具响应会以用户消息的形式返回
      - 格式：\`\`\`json:mcp-response:{clientId}\`\`\`
      - 在发起下一次工具调用前，必须先等待并处理当前工具的响应

   C. 重要规则（Important Rules）:
      - 只能使用 "tools/call" 作为 method
      - 每条消息 **只能包含一个** 工具调用
      - 要**实际行动**，不要只是口头描述你会调用哪些工具
      - 在代码块语言标签中填入正确的 clientId
      - 确认传入的 arguments 与 primitive 的参数要求完全匹配

4. 交互流程（INTERACTION FLOW）:
   A. 当用户发出请求时：
      - 如果有合适的工具，应**立即**调用
      - 不要先询问用户「是否需要使用工具」
      - 不要只说明你「可以」调用哪些工具，而是直接去调用
   B. 在收到工具响应后：
      - 用自然语言向用户解释工具结果
      - 如有必要，继续发起下一步合适的工具调用
   C. 当工具调用失败时：
      - 清晰说明错误原因或错误信息
      - 立即尝试其它可行的替代方案

5. 交互示例（EXAMPLE INTERACTION）:

  正确示例（good example）:

   \`\`\`json:mcp:filesystem
   {
     "method": "tools/call",
     "params": {
       "name": "list_allowed_directories",
       "arguments": {}
     }
   }
   \`\`\`"


  \`\`\`json:mcp-response:filesystem
  {
    "method": "tools/call",
    "params": {
      "name": "write_file",
      "arguments": {
        "path": "/Users/river/dev/nextchat/test/joke.txt",
        "content": "为什么数学书总是感到忧伤？因为它有太多的问题。"
      }
    }
  }
  \`\`\`

  错误示例（following is the wrong mcp json example）:

   \`\`\`json:mcp:filesystem
   {
      "method": "write_file",
      "params": {
        "path": "NextChat_Information.txt",
        "content": "1"
      }
   }
   \`\`\`

   上面的示例是错误的，因为 method 不是 "tools/call"。
   
   \`\`\`json
   {
     "method": "search_repositories",
     "params": {
       "query": "2oeee"
     }
   }
   \`\`\`

   这同样是错误的，因为 method 不是 "tools/call"。!!!!!!!!!!!

   正确写法应为（the right format is）:
   \`\`\`json:mcp:filesystem
   {
     "method": "tools/call",
     "params": {
       "name": "search_repositories",
       "arguments": {
         "query": "2oeee"
       }
     }
   }
   \`\`\`
   
   请严格遵守以上格式，**只使用 "tools/call" 这一种 method** 进行 MCP 调用！！！！！！！！
   
`;

export const SUMMARIZE_MODEL = "gpt-4o-mini";
export const GEMINI_SUMMARIZE_MODEL = "gemini-pro";
export const DEEPSEEK_SUMMARIZE_MODEL = "deepseek-chat";

export const KnowledgeCutOffDate: Record<string, string> = {
  default: "2021-09",
  "gpt-4-turbo": "2023-12",
  "gpt-4-turbo-2024-04-09": "2023-12",
  "gpt-4-turbo-preview": "2023-12",
  "gpt-4.1": "2024-06",
  "gpt-4.1-2025-04-14": "2024-06",
  "gpt-4.1-mini": "2024-06",
  "gpt-4.1-mini-2025-04-14": "2024-06",
  "gpt-4.1-nano": "2024-06",
  "gpt-4.1-nano-2025-04-14": "2024-06",
  "gpt-4.5-preview": "2023-10",
  "gpt-4.5-preview-2025-02-27": "2023-10",
  "gpt-4o": "2023-10",
  "gpt-4o-2024-05-13": "2023-10",
  "gpt-4o-2024-08-06": "2023-10",
  "gpt-4o-2024-11-20": "2023-10",
  "chatgpt-4o-latest": "2023-10",
  "gpt-4o-mini": "2023-10",
  "gpt-4o-mini-2024-07-18": "2023-10",
  "gpt-4-vision-preview": "2023-04",
  "o1-mini-2024-09-12": "2023-10",
  "o1-mini": "2023-10",
  "o1-preview-2024-09-12": "2023-10",
  "o1-preview": "2023-10",
  "o1-2024-12-17": "2023-10",
  o1: "2023-10",
  "o3-mini-2025-01-31": "2023-10",
  "o3-mini": "2023-10",
  // After improvements,
  // it's now easier to add "KnowledgeCutOffDate" instead of stupid hardcoding it, as was done previously.
  "gemini-pro": "2023-12",
  "gemini-pro-vision": "2023-12",
  "deepseek-chat": "2024-07",
  "deepseek-coder": "2024-07",
};

export const DEFAULT_TTS_ENGINE = ServiceProvider.OpenAI;
export const DEFAULT_TTS_MODEL = "tts-1";
export const DEFAULT_TTS_VOICE = "alloy";

export const OPENAI_TTS = {
    Provider: ServiceProvider.OpenAI,
    ModelProvider: ModelProvider.GPT,
    Model: ["tts-1", "tts-1-hd"],
    Voices: ["alloy", "echo", "fable", "onyx", "nova", "shimmer"],
} as const;

export const ALIBABA_TTS = {
    Provider: ServiceProvider.Alibaba,
    ModelProvider: ModelProvider.Qwen,
    Model: ["qwen3-tts-flash", "qwen3-tts-flash-2025-11-27"],
    Voices: ['Cherry', 'Serena', 'Ethan', 'Chelsie', 'Momo', 'Vivian', 'Moon', 'Maia', 'Kai', 'Notish', 'Bella', 'Jennifer', 'Ryan', 'Katerina', 'Aiden', 'Eldric Sage', 'Mia', 'Mochi', 'Bellona', 'Vincent', 'Bunny', 'Neil', 'Elias', 'Arthur', 'Nini', 'Ebony', 'Seren', 'Pip', 'Stella', 'Bodega', 'Sonrisa', 'Alek', 'Dolce', 'Sohee', 'Ono Anna', 'Lenn', 'Emilien', 'Andre', 'Radio Gol', 'Jada', 'Dylan', 'Li', 'Marcus', 'Roy', 'Peter', 'Sunny', 'Eric', 'Rocky', 'Kiki'],
} as const;

export const EDGE_TTS = {
    Provider: "Edge" as const,
    ModelProvider: ModelProvider.GPT,
    Model: [] as string[],
    Voices: [] as string[],
} as const;

export type TTSEngineType = ServiceProvider.OpenAI | ServiceProvider.Alibaba | "Edge";

export const DEFAULT_TTS_ENGINES = [ServiceProvider.OpenAI, ServiceProvider.Alibaba, "Edge"] as const;
export const DEFAULT_TTS_MODELS = [...OPENAI_TTS.Model, ...ALIBABA_TTS.Model] as const;
export const DEFAULT_TTS_VOICES = [...OPENAI_TTS.Voices, ...ALIBABA_TTS.Voices] as const;

interface TTSConfigItem {
    Provider: ServiceProvider | "Edge";
    Model: readonly string[];
    Voices: readonly string[];
    ModelProvider: ModelProvider;
}

export const TTS_CONFIGS: Record<TTSEngineType, TTSConfigItem> = {
    [ServiceProvider.OpenAI]: OPENAI_TTS,
    [ServiceProvider.Alibaba]: ALIBABA_TTS,
    Edge: EDGE_TTS,
} as const;

export const VISION_MODEL_REGEXES = [
  /vision/,
  /gpt-4o/,
  /gpt-4\.1/,
  /claude.*[34]/,
  /gemini-1\.5/,
  /gemini-exp/,
  /gemini-2\.[05]/,
  /learnlm/,
  /qwen-vl/,
  /qwen3-vl/,
  /omni/i,
  /image/i,
  /qvq/,
  /gpt-4-turbo(?!.*preview)/,
  /^dall-e-3$/,
  /glm-4v/,
  /vl/i,
  /o3/,
  /o4-mini/,
  /grok-4/i,
  /gpt-5/
];

export const EXCLUDE_VISION_MODEL_REGEXES = [/claude-3-5-haiku-20241022/];

const openaiModels = [
  // As of July 2024, gpt-4o-mini should be used in place of gpt-3.5-turbo,
  // as it is cheaper, more capable, multimodal, and just as fast. gpt-3.5-turbo is still available for use in the API.
  "gpt-3.5-turbo",
  "gpt-3.5-turbo-1106",
  "gpt-3.5-turbo-0125",
  "gpt-4",
  "gpt-4-0613",
  "gpt-4-32k",
  "gpt-4-32k-0613",
  "gpt-4-turbo",
  "gpt-4-turbo-preview",
  "gpt-4.1",
  "gpt-4.1-2025-04-14",
  "gpt-4.1-mini",
  "gpt-4.1-mini-2025-04-14",
  "gpt-4.1-nano",
  "gpt-4.1-nano-2025-04-14",
  "gpt-4.5-preview",
  "gpt-4.5-preview-2025-02-27",
  "gpt-5-chat",
  "gpt-5-mini",
  "gpt-5-nano",
  "gpt-5",
  "gpt-5-chat-2025-01-01-preview",
  "gpt-4o",
  "gpt-4o-2024-05-13",
  "gpt-4o-2024-08-06",
  "gpt-4o-2024-11-20",
  "chatgpt-4o-latest",
  "gpt-4o-mini",
  "gpt-4o-mini-2024-07-18",
  "gpt-4-vision-preview",
  "gpt-4-turbo-2024-04-09",
  "gpt-4-1106-preview",
  "dall-e-3",
  "o1-mini",
  "o1-preview",
  "o3-mini",
  "o3",
  "o4-mini",
];

const googleModels = [
  "gemini-1.5-pro-latest",
  "gemini-1.5-pro",
  "gemini-1.5-pro-002",
  "gemini-1.5-flash-latest",
  "gemini-1.5-flash-8b-latest",
  "gemini-1.5-flash",
  "gemini-1.5-flash-8b",
  "gemini-1.5-flash-002",
  "learnlm-1.5-pro-experimental",
  "gemini-exp-1206",
  "gemini-2.0-flash",
  "gemini-2.0-flash-exp",
  "gemini-2.0-flash-lite-preview-02-05",
  "gemini-2.0-flash-thinking-exp",
  "gemini-2.0-flash-thinking-exp-1219",
  "gemini-2.0-flash-thinking-exp-01-21",
  "gemini-2.0-pro-exp",
  "gemini-2.0-pro-exp-02-05",
  "gemini-2.5-pro-preview-06-05",
  "gemini-2.5-pro"
];

const anthropicModels = [
  "claude-instant-1.2",
  "claude-2.0",
  "claude-2.1",
  "claude-3-sonnet-20240229",
  "claude-3-opus-20240229",
  "claude-3-opus-latest",
  "claude-3-haiku-20240307",
  "claude-3-5-haiku-20241022",
  "claude-3-5-haiku-latest",
  "claude-3-5-sonnet-20240620",
  "claude-3-5-sonnet-20241022",
  "claude-3-5-sonnet-latest",
  "claude-3-7-sonnet-20250219",
  "claude-3-7-sonnet-latest",
  "claude-sonnet-4-20250514",
  "claude-opus-4-20250514",
];

const baiduModels = [
  "ernie-4.0-turbo-8k",
  "ernie-4.0-8k",
  "ernie-4.0-8k-preview",
  "ernie-4.0-8k-preview-0518",
  "ernie-4.0-8k-latest",
  "ernie-3.5-8k",
  "ernie-3.5-8k-0205",
  "ernie-speed-128k",
  "ernie-speed-8k",
  "ernie-lite-8k",
  "ernie-tiny-8k",
];

const bytedanceModels = [
  "Doubao-lite-4k",
  "Doubao-lite-32k",
  "Doubao-lite-128k",
  "Doubao-pro-4k",
  "Doubao-pro-32k",
  "Doubao-pro-128k",
];

const alibabaModes = [
  "qwen-turbo-latest",
  "qwen-plus-latest",
  "qwen3-max",
  "qwen3-max-preview",
  "qwen3-next-80b-a3b-thinking",
  "qwen3-coder-plus",
  "deepseek-v3.2",
  "deepseek-r1",
  "Moonshot-Kimi-K2-Instruct",
  "kimi-k2-thinking",
  "qvq-max-latest",
  "qwen3-omni-flash",
  "qwen3-vl-plus",
  "glm-4.7"
];

const miniMaxModels = [
    "MiniMax-M2.1",
    "MiniMax-M2.1-lightning"
]

const tencentModels = [
  "hunyuan-pro",
  "hunyuan-standard",
  "hunyuan-lite",
  "hunyuan-role",
  "hunyuan-functioncall",
  "hunyuan-code",
  "hunyuan-vision",
];

const moonshotModels = [
  "moonshot-v1-auto",
  "moonshot-v1-8k",
  "moonshot-v1-32k",
  "moonshot-v1-128k",
  "moonshot-v1-8k-vision-preview",
  "moonshot-v1-32k-vision-preview",
  "moonshot-v1-128k-vision-preview",
  "kimi-thinking-preview",
  "kimi-k2-0711-preview",
  "kimi-latest",
];

const iflytekModels = [
  "general",
  "generalv3",
  "pro-128k",
  "generalv3.5",
  "4.0Ultra",
];

const deepseekModels = ["deepseek-chat", "deepseek-coder", "deepseek-reasoner"];

const xAIModes = [
  "grok-beta",
  "grok-2",
  "grok-2-1212",
  "grok-2-latest",
  "grok-vision-beta",
  "grok-2-vision-1212",
  "grok-2-vision",
  "grok-2-vision-latest",
  "grok-3-mini-fast-beta",
  "grok-3-mini-fast",
  "grok-3-mini-fast-latest",
  "grok-3-mini-beta",
  "grok-3-mini",
  "grok-3-mini-latest",
  "grok-3-fast-beta",
  "grok-3-fast",
  "grok-3-fast-latest",
  "grok-3-beta",
  "grok-3",
  "grok-3-latest",
];

const chatglmModels = [
  "glm-4-plus",
  "glm-4-0520",
  "glm-4",
  "glm-4-air",
  "glm-4-airx",
  "glm-4-long",
  "glm-4-flashx",
  "glm-4-flash",
  "glm-4v-plus",
  "glm-4v",
  "glm-4v-flash", // free
  "cogview-3-plus",
  "cogview-3",
  "cogview-3-flash", // free
  // 目前无法适配轮询任务
  //   "cogvideox",
  //   "cogvideox-flash", // free
];

const siliconflowModels = [
  "Qwen/Qwen2.5-7B-Instruct",
  "Qwen/Qwen2.5-72B-Instruct",
  "deepseek-ai/DeepSeek-R1",
  "deepseek-ai/DeepSeek-R1-Distill-Llama-70B",
  "deepseek-ai/DeepSeek-R1-Distill-Llama-8B",
  "deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B",
  "deepseek-ai/DeepSeek-R1-Distill-Qwen-14B",
  "deepseek-ai/DeepSeek-R1-Distill-Qwen-32B",
  "deepseek-ai/DeepSeek-R1-Distill-Qwen-7B",
  "deepseek-ai/DeepSeek-V3",
  "meta-llama/Llama-3.3-70B-Instruct",
  "THUDM/glm-4-9b-chat",
  "Pro/deepseek-ai/DeepSeek-R1",
  "Pro/deepseek-ai/DeepSeek-V3",
];

const ai302Models = [
  "deepseek-chat",
  "gpt-4o",
  "chatgpt-4o-latest",
  "llama3.3-70b",
  "deepseek-reasoner",
  "gemini-2.0-flash",
  "claude-3-7-sonnet-20250219",
  "claude-3-7-sonnet-latest",
  "grok-3-beta",
  "grok-3-mini-beta",
  "gpt-4.1",
  "gpt-4.1-mini",
  "o3",
  "o4-mini",
  "qwen3-235b-a22b",
  "qwen3-32b",
  "gemini-2.5-pro-preview-05-06",
  "llama-4-maverick",
  "gemini-2.5-flash",
  "claude-sonnet-4-20250514",
  "claude-opus-4-20250514",
  "gemini-2.5-pro",
];

let seq = 1000; // 内置的模型序号生成器从1000开始
export const DEFAULT_MODELS = [
  //   ...openaiModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++, // Global sequence sort(index)
  //     provider: {
  //       id: "openai",
  //       providerName: "OpenAI",
  //       providerType: "openai",
  //       sorted: 1, // 这里是固定的，确保顺序与之前内置的版本一致
  //     },
  //   })),
  //   ...openaiModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "azure",
  //       providerName: "Azure",
  //       providerType: "azure",
  //       sorted: 2,
  //     },
  //   })),
  //   ...googleModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "google",
  //       providerName: "Google",
  //       providerType: "google",
  //       sorted: 3,
  //     },
  //   })),
  //   ...anthropicModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "anthropic",
  //       providerName: "Anthropic",
  //       providerType: "anthropic",
  //       sorted: 4,
  //     },
  //   })),
  //   ...baiduModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "baidu",
  //       providerName: "Baidu",
  //       providerType: "baidu",
  //       sorted: 5,
  //     },
  //   })),
  //   ...bytedanceModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "bytedance",
  //       providerName: "ByteDance",
  //       providerType: "bytedance",
  //       sorted: 6,
  //     },
  //   })),
  ...alibabaModes.map((name) => ({
    name,
    available: true,
    sorted: seq++,
    provider: {
      id: "alibaba",
      providerName: "Alibaba",
      providerType: "alibaba",
      sorted: 7,
    },
  })),
  ...miniMaxModels.map((name) => ({
    name,
    available: true,
    sorted: seq++,
    provider: {
      id: "miniMax",
      providerName: "MiniMax",
      providerType: "miniMax",
      sorted: 8,
    },
  })),
  //   ...tencentModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "tencent",
  //       providerName: "Tencent",
  //       providerType: "tencent",
  //       sorted: 8,
  //     },
  //   })),
  //   ...moonshotModes.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "moonshot",
  //       providerName: "Moonshot",
  //       providerType: "moonshot",
  //       sorted: 9,
  //     },
  //   })),
  //   ...iflytekModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "iflytek",
  //       providerName: "Iflytek",
  //       providerType: "iflytek",
  //       sorted: 10,
  //     },
  //   })),
  //   ...xAIModes.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "xai",
  //       providerName: "XAI",
  //       providerType: "xai",
  //       sorted: 11,
  //     },
  //   })),
  //   ...chatglmModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "chatglm",
  //       providerName: "ChatGLM",
  //       providerType: "chatglm",
  //       sorted: 12,
  //     },
  //   })),
    ...deepseekModels.map((name) => ({
      name,
      available: true,
      sorted: seq++,
      provider: {
        id: "deepseek",
        providerName: "DeepSeek",
        providerType: "deepseek",
        sorted: 13,
      },
    })),
  //   ...siliconflowModels.map((name) => ({
  //     name,
  //     available: true,
  //     sorted: seq++,
  //     provider: {
  //       id: "siliconflow",
  //       providerName: "SiliconFlow",
  //       providerType: "siliconflow",
  //       sorted: 14,
  //     },
  //   })),
] as const;

export const CHAT_PAGE_SIZE = 15;
export const MAX_RENDER_MSG_COUNT = 45;

// some famous webdav endpoints
export const internalAllowedWebDavEndpoints = [
  "https://dav.jianguoyun.com/dav/",
  "https://dav.dropdav.com/",
  "https://dav.box.com/dav",
  "https://nanao.teracloud.jp/dav/",
  "https://bora.teracloud.jp/dav/",
  "https://webdav.4shared.com/",
  "https://dav.idrivesync.com",
  "https://webdav.yandex.com",
  "https://app.koofr.net/dav/Koofr",
];

export const DEFAULT_GA_ID = "G-89WN60ZK2E";

export const SAAS_CHAT_URL = "https://nextchat.club";
export const SAAS_CHAT_UTM_URL = "https://nextchat.club?utm=github";

