<!-- AI 助手应用：通过 Electron IPC 安全调用本机 Codex SDK -->
<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { api, type AIMessage } from "@/lib/api";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";

const { text } = useInterfaceLanguage();
const props = defineProps<{ sessionId: string }>();
const messages = ref<AIMessage[]>([]);
const draft = ref("");
const isSending = ref(false);
const error = ref("");
const model = ref("");
const messageList = ref<HTMLElement | null>(null);
const conversationId = ref(crypto.randomUUID());
const quickActions = computed(() => [
  {
    icon: "icon-[lucide--activity]",
    label: text("Analyze server health", "分析服务器健康状态"),
    prompt: text(
      "Analyze the current server health. Prioritize any risks and give me the safest next steps.",
      "分析当前服务器健康状态，按优先级指出风险，并给出最安全的下一步操作。",
    ),
    diagnostics: true,
  },
  {
    icon: "icon-[lucide--cpu]",
    label: text("Investigate high CPU", "排查 CPU 过高"),
    prompt: text(
      "Investigate CPU pressure using the current diagnostics and identify the most likely causes.",
      "根据当前诊断数据排查 CPU 压力，并指出最可能的原因。",
    ),
    diagnostics: true,
  },
  {
    icon: "icon-[lucide--database]",
    label: text("Database troubleshooting", "数据库故障排查"),
    prompt: text(
      "Give me a safe, step-by-step database troubleshooting checklist. Ask which database engine I use if needed.",
      "给我一份安全、分步骤的数据库故障排查清单，如有需要先询问数据库类型。",
    ),
    diagnostics: false,
  },
  {
    icon: "icon-[lucide--shield-check]",
    label: text("Security review", "安全检查"),
    prompt: text(
      "Review the current server diagnostics for security and reliability risks. Do not suggest destructive changes without a warning.",
      "根据当前服务器诊断数据检查安全性和可靠性风险，涉及破坏性修改时必须明确警告。",
    ),
    diagnostics: true,
  },
]);

async function scrollToLatest() {
  await nextTick();
  messageList.value?.scrollTo({ top: messageList.value.scrollHeight, behavior: "smooth" });
}

async function requestAI(content: string, serverContext?: string) {
  if (!content || isSending.value) return;

  const userMessage: AIMessage = { role: "user", content };
  messages.value.push(userMessage);
  draft.value = "";
  error.value = "";
  isSending.value = true;
  await scrollToLatest();

  try {
    const response = await api.chatWithAI(messages.value, serverContext, conversationId.value);
    messages.value.push({ role: "assistant", content: response.text });
    model.value = response.model;
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause);
  } finally {
    isSending.value = false;
    await scrollToLatest();
  }
}

async function sendMessage() {
  const content = draft.value.trim();
  await requestAI(content);
}

async function runQuickAction(action: (typeof quickActions.value)[number]) {
  let serverContext: string | undefined;
  if (action.diagnostics) {
    isSending.value = true;
    error.value = "";
    try {
      const stats = await api.getSystemStats(props.sessionId);
      serverContext = JSON.stringify(stats);
    } catch (cause) {
      error.value = text(
        `Unable to collect server diagnostics: ${cause instanceof Error ? cause.message : String(cause)}`,
        `无法获取服务器诊断数据：${cause instanceof Error ? cause.message : String(cause)}`,
      );
      isSending.value = false;
      return;
    }
    isSending.value = false;
  }
  await requestAI(action.prompt, serverContext);
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    void sendMessage();
  }
}

function clearConversation() {
  messages.value = [];
  error.value = "";
  model.value = "";
  conversationId.value = crypto.randomUUID();
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col bg-surface text-primary">
    <header class="flex h-14 shrink-0 items-center justify-between border-b border-subtle px-5">
      <div class="flex items-center gap-3">
        <div
          class="grid size-9 place-items-center rounded-[10px] bg-gradient-to-br from-violet-500 to-blue-500 shadow-lg"
        >
          <span class="icon-[lucide--sparkles] size-5 text-white"></span>
        </div>
        <div>
          <h2 class="text-sm font-semibold">SynapSH AI</h2>
          <p class="text-[11px] text-tertiary">
            {{ text("Server operations assistant", "服务器运维助手") }}
            <span v-if="model"> · {{ model }}</span>
          </p>
        </div>
      </div>
      <button
        type="button"
        class="app-btn app-btn-secondary min-h-8 px-3 text-xs"
        :disabled="messages.length === 0"
        @click="clearConversation"
      >
        <span class="icon-[lucide--trash-2] size-3.5"></span>
        {{ text("Clear", "清空") }}
      </button>
    </header>

    <main ref="messageList" class="custom-scrollbar flex-1 overflow-y-auto px-5 py-6">
      <div
        v-if="messages.length === 0"
        class="mx-auto flex h-full max-w-lg flex-col items-center justify-center text-center"
      >
        <div class="mb-5 grid size-16 place-items-center rounded-2xl bg-accent/10 text-accent">
          <span class="icon-[lucide--bot] size-8"></span>
        </div>
        <h3 class="mb-2 text-base font-semibold">
          {{ text("How can I help?", "有什么可以帮你？") }}
        </h3>
        <p class="max-w-sm text-xs leading-5 text-tertiary">
          {{
            text(
              "Ask about Linux commands, SSH issues, databases, scripts, or server troubleshooting.",
              "可以询问 Linux 命令、SSH 问题、数据库、脚本或服务器故障排查。",
            )
          }}
        </p>
        <div class="mt-6 grid w-full max-w-xl grid-cols-2 gap-2">
          <button
            v-for="action in quickActions"
            :key="action.label"
            type="button"
            class="flex min-h-11 items-center gap-2 rounded-xl border border-subtle bg-elevated px-3 text-left text-xs text-secondary transition-all hover:border-accent/40 hover:text-primary"
            :disabled="isSending"
            @click="runQuickAction(action)"
          >
            <span :class="action.icon" class="size-4 shrink-0 text-accent"></span>
            {{ action.label }}
          </button>
        </div>
      </div>

      <div v-else class="mx-auto flex max-w-3xl flex-col gap-4">
        <article
          v-for="(message, index) in messages"
          :key="index"
          class="flex gap-3"
          :class="message.role === 'user' ? 'justify-end' : 'justify-start'"
        >
          <div
            v-if="message.role === 'assistant'"
            class="mt-0.5 grid size-7 shrink-0 place-items-center rounded-lg bg-violet-500/15 text-violet-400"
          >
            <span class="icon-[lucide--sparkles] size-4"></span>
          </div>
          <div
            class="max-w-[78%] whitespace-pre-wrap rounded-2xl px-4 py-3 text-[13px] leading-6"
            :class="
              message.role === 'user'
                ? 'rounded-br-md bg-accent text-white'
                : 'rounded-bl-md border border-subtle bg-elevated text-primary'
            "
          >
            {{ message.content }}
          </div>
        </article>

        <div v-if="isSending" class="flex items-center gap-3 text-tertiary">
          <div class="grid size-7 place-items-center rounded-lg bg-violet-500/15 text-violet-400">
            <span class="icon-[lucide--sparkles] size-4"></span>
          </div>
          <div
            class="flex gap-1 rounded-2xl rounded-bl-md border border-subtle bg-elevated px-4 py-3"
          >
            <span
              v-for="dot in 3"
              :key="dot"
              class="size-1.5 animate-pulse rounded-full bg-tertiary"
            ></span>
          </div>
        </div>
      </div>
    </main>

    <div
      v-if="error"
      class="mx-5 mb-3 rounded-lg border border-danger/20 bg-danger/10 px-3 py-2 text-xs text-danger"
    >
      {{ error }}
    </div>

    <footer class="shrink-0 border-t border-subtle bg-surface/90 p-4 backdrop-blur-xl">
      <div
        class="mx-auto flex max-w-3xl items-end gap-2 rounded-2xl border border-subtle bg-elevated p-2 focus-within:border-accent/50"
      >
        <textarea
          v-model="draft"
          rows="1"
          class="max-h-32 min-h-9 flex-1 resize-none bg-transparent px-2 py-2 text-[13px] leading-5 outline-none placeholder:text-tertiary"
          :placeholder="text('Ask SynapSH AI...', '询问 SynapSH AI...')"
          :disabled="isSending"
          @keydown="handleKeydown"
        ></textarea>
        <button
          type="button"
          class="grid size-9 shrink-0 place-items-center rounded-xl bg-accent text-white transition-all hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-40"
          :disabled="!draft.trim() || isSending"
          :aria-label="text('Send message', '发送消息')"
          @click="sendMessage"
        >
          <span class="icon-[lucide--arrow-up] size-4"></span>
        </button>
      </div>
      <p class="mt-2 text-center text-[10px] text-tertiary/70">
        {{
          text("AI can make mistakes. Verify important commands.", "AI 可能出错，请核对重要命令。")
        }}
      </p>
    </footer>
  </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 5px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: var(--border-strong);
}
</style>
