"use client";

import { ApiPath, MINIMAX_BASE_URL, MiniMax } from "@/app/constant";
import {
  useAccessStore,
  useAppConfig,
  useChatStore,
  ChatMessageTool,
  usePluginStore,
} from "@/app/store";
import { streamWithThink } from "@/app/utils/chat";
import {
  ChatOptions,
  getHeaders,
  LLMApi,
  LLMModel,
  SpeechOptions,
} from "../api";
import { getClientConfig } from "@/app/config/client";
import {
  getMessageTextContent,
  getMessageTextContentWithoutThinking,
  getTimeoutMSByModel,
} from "@/app/utils";
import { RequestPayload } from "./openai";
import { fetch } from "@/app/utils/stream";

export class MiniMaxApi implements LLMApi {
  private disableListModels = true;

  path(path: string): string {
    const accessStore = useAccessStore.getState();

    let baseUrl = "";

    if (accessStore.useCustomConfig) {
      baseUrl = accessStore.minimaxUrl;
    }

    if (baseUrl.length === 0) {
      const isApp = !!getClientConfig()?.isApp;
      const apiPath = ApiPath.MiniMax;
      baseUrl = isApp ? MINIMAX_BASE_URL : apiPath;
    }

    if (baseUrl.endsWith("/")) {
      baseUrl = baseUrl.slice(0, baseUrl.length - 1);
    }
    if (!baseUrl.startsWith("http") && !baseUrl.startsWith(ApiPath.MiniMax)) {
      baseUrl = "https://" + baseUrl;
    }

    console.log("[Proxy Endpoint] ", baseUrl, path);

    return [baseUrl, path].join("/");
  }

  extractMessage(res: any) {
    return res.choices?.at(0)?.message?.content ?? "";
  }

  speech(options: SpeechOptions): Promise<ArrayBuffer> {
    throw new Error("Method not implemented.");
  }

  async chat(options: ChatOptions) {
    const messages: ChatOptions["messages"] = [];
    for (const v of options.messages) {
      if (v.role === "assistant") {
        const content = getMessageTextContentWithoutThinking(v);
        messages.push({ role: v.role, content });
      } else {
        const content = getMessageTextContent(v);
        messages.push({ role: v.role, content });
      }
    }

    // 检测并修复消息顺序，确保除system外的第一个消息是user
    const filteredMessages: ChatOptions["messages"] = [];
    let hasFoundFirstUser = false;

    for (const msg of messages) {
      if (msg.role === "system") {
        // Keep all system messages
        filteredMessages.push(msg);
      } else if (msg.role === "user") {
        // User message directly added
        filteredMessages.push(msg);
        hasFoundFirstUser = true;
      } else if (hasFoundFirstUser) {
        // After finding the first user message, all subsequent non-system messages are retained.
        filteredMessages.push(msg);
      }
      // If hasFoundFirstUser is false and it is not a system message, it will be skipped.
    }

    const modelConfig = {
      ...useAppConfig.getState().modelConfig,
      ...useChatStore.getState().currentSession().mask.modelConfig,
      ...{
        model: options.config.model,
        providerName: options.config.providerName,
      },
    };

    const requestPayload: RequestPayload = {
      messages: filteredMessages,
      stream: options.config.stream,
      model: modelConfig.model,
      temperature: modelConfig.temperature,
      presence_penalty: modelConfig.presence_penalty,
      frequency_penalty: modelConfig.frequency_penalty,
      top_p: modelConfig.top_p,
    };

    console.log("[Request] MiniMax payload: ", requestPayload);

    const shouldStream = !!options.config.stream;
    const controller = new AbortController();
    options.onController?.(controller);

    try {
      const chatPath = this.path(MiniMax.ChatPath);
      const chatPayload = {
        method: "POST",
        body: JSON.stringify(requestPayload),
        signal: controller.signal,
        headers: getHeaders(false, options.config.providerName),
      };

      // make a fetch request
      const requestTimeoutId = setTimeout(
        () => controller.abort(),
        getTimeoutMSByModel(options.config.model),
      );

      if (shouldStream) {
        const [tools, funcs] = usePluginStore
          .getState()
          .getAsTools(
            useChatStore.getState().currentSession().mask?.plugin || [],
          );

        // State for tracking MiniMax think content across chunks
        let isInThink = false;
        let thinkContent = "";
        let pendingAnswer = "";

        return streamWithThink(
          chatPath,
          requestPayload,
          getHeaders(false, options.config.providerName),
          tools as any,
          funcs,
          controller,
          // parseSSE
          (text: string, runTools: ChatMessageTool[]) => {
            // MiniMax SSE format: data:{"id":"...","choices":[...]}
            // Need to strip "data:" prefix if present
            let jsonText = text;
            if (text.startsWith("data:")) {
              jsonText = text.slice(5).trim();
            }

            // If we have pending answer from previous chunk (e.g. answer after </think>),
            // DO NOT early-return here. Otherwise we'll drop the current chunk content.
            let pendingAnswerPrefix = "";
            if (pendingAnswer.length > 0) {
              pendingAnswerPrefix = pendingAnswer;
              pendingAnswer = "";
            }

            // Handle empty / done SSE messages
            // NOTE: must be AFTER pendingAnswer flush, otherwise answer that is queued at </think>
            // could be swallowed by a trailing [DONE] event.
            if (!jsonText || jsonText === "[DONE]") {
              return {
                isThinking: false,
                content: pendingAnswerPrefix,
              };
            }

            const json = JSON.parse(jsonText);
            const choices = json.choices as Array<{
              delta: {
                content: string | null;
                tool_calls: ChatMessageTool[];
                reasoning_content: string | null;
              };
            }>;
            const tool_calls = choices[0]?.delta?.tool_calls;
            if (tool_calls?.length > 0) {
              const index = tool_calls[0]?.index;
              const id = tool_calls[0]?.id;
              const args = tool_calls[0]?.function?.arguments;
              if (id) {
                runTools.push({
                  id,
                  type: tool_calls[0]?.type,
                  function: {
                    name: tool_calls[0]?.function?.name as string,
                    arguments: args,
                  },
                });
              } else {
                // @ts-ignore
                runTools[index]["function"]["arguments"] += args;
              }
            }
            const reasoning = choices[0]?.delta?.reasoning_content;
            let content = choices[0]?.delta?.content;

            // Skip if both content and reasoning are empty or null
            if (
              (!reasoning || reasoning.length === 0) &&
              (!content || content.length === 0)
            ) {
              return {
                isThinking: false,
                content: pendingAnswerPrefix,
              };
            }

            // Handle reasoning content if present (older format)
            if (reasoning && reasoning.length > 0) {
              // If we somehow have pending answer prefix, prefer showing answer first.
              if (pendingAnswerPrefix.length > 0) {
                // Put reasoning back to buffer (best-effort) so it can be rendered next chunk.
                thinkContent = reasoning + thinkContent;
                return { isThinking: false, content: pendingAnswerPrefix };
              }
              return {
                isThinking: true,
                content: reasoning,
              };
            }

            // Process MiniMax think tags
            if (content && content.length > 0) {
              // Check if we're currently inside a think section
              if (!isInThink && content.includes("<think>")) {
                isInThink = true;
                thinkContent = "";
                // Remove everything before <think> (should be none)
                const thinkStart = content.indexOf("<think>");
                content = content
                  .slice(thinkStart + "<think>".length)
                  .trimStart();
              }

              if (isInThink) {
                // Check if think ends in this chunk
                if (content.includes("</think>")) {
                  const thinkEnd = content.indexOf("</think>");
                  // Add content before </think> to thinkContent
                  const thinkDelta = content.slice(0, thinkEnd);
                  thinkContent += thinkDelta;
                  // Content after </think> is answer
                  const answerContent = content.slice(
                    thinkEnd + "</think>".length,
                  );
                  // Reset think state
                  isInThink = false;
                  // keep thinkContent for debugging if needed; don't emit the whole buffer to UI
                  // (emitting full buffer causes duplicated "thinking" text on the UI side)
                  thinkContent = "";

                  // Store answer content as pending for next chunk
                  if (answerContent.length > 0) {
                    pendingAnswer = answerContent.trimStart();
                  }
                  // Return ONLY the delta from this chunk to avoid duplication
                  // If we have pending answer prefix, show it first to avoid mixing channels.
                  if (pendingAnswerPrefix.length > 0) {
                    // best-effort: queue current think delta for later
                    thinkContent = thinkDelta + thinkContent;
                    return { isThinking: false, content: pendingAnswerPrefix };
                  }
                  return {
                    isThinking: true,
                    content: thinkDelta,
                  };
                } else {
                  // Still in think, no </think> yet
                  thinkContent += content;
                  if (pendingAnswerPrefix.length > 0) {
                    // avoid mixing answer prefix into thinking channel
                    return { isThinking: false, content: pendingAnswerPrefix };
                  }
                  return {
                    isThinking: true,
                    content: content,
                  };
                }
              } else {
                // Not in think, this is regular answer content
                return {
                  isThinking: false,
                  content: pendingAnswerPrefix + content,
                };
              }
            }

            return {
              isThinking: false,
              content: pendingAnswerPrefix,
            };
          },
          // processToolMessage
          (
            requestPayload: RequestPayload,
            toolCallMessage: any,
            toolCallResult: any[],
          ) => {
            // @ts-ignore
            requestPayload?.messages?.splice(
              // @ts-ignore
              requestPayload?.messages?.length,
              0,
              toolCallMessage,
              ...toolCallResult,
            );
          },
          options,
        );
      } else {
        const res = await fetch(chatPath, chatPayload);
        clearTimeout(requestTimeoutId);

        const resJson = await res.json();
        const message = this.extractMessage(resJson);
        options.onFinish(message, res);
      }
    } catch (e) {
      console.log("[Request] failed to make a chat request", e);
      options.onError?.(e as Error);
    }
  }

  async usage() {
    return {
      used: 0,
      total: 0,
    };
  }

  async models(): Promise<LLMModel[]> {
    return [];
  }
}
