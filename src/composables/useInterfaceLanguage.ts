// 全局界面语言状态，负责中英文切换与本地持久化
import { computed, watch } from "vue";
import { useLocalStorage } from "@vueuse/core";

export type InterfaceLanguage = "en-US" | "zh-CN";

const language = useLocalStorage<InterfaceLanguage>("general-language", "en-US");

watch(
  language,
  (value) => {
    document.documentElement.lang = value;
  },
  { immediate: true },
);

export function useInterfaceLanguage() {
  const isEnglish = computed(() => language.value === "en-US");
  const text = (english: string, chinese: string) =>
    language.value === "en-US" ? english : chinese;

  return {
    language,
    isEnglish,
    text,
  };
}
