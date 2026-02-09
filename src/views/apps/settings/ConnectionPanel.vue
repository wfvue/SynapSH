<!--
  ConnectionPanel.vue - 连接设置面板
  设置 SSH 连接默认配置
-->
<script setup lang="ts">
import { ref } from "vue";
import { Switch } from "@/components/ui/switch";

// 连接设置项
const timeout = ref(30);
const keepAlive = ref(60);
const compression = ref(false);
const sshVersion = ref("2");
const authMethod = ref("password");

const authMethods = [
    { value: "password", label: "密码", icon: "icon-[mdi--key]" },
    { value: "publickey", label: "密钥", icon: "icon-[mdi--key-chain]" },
    { value: "both", label: "密码 + 密钥", icon: "icon-[mdi--shield-key]" },
];
</script>

<template>
    <div class="p-8 max-w-5xl mx-auto text-foreground animate-in fade-in duration-500">
        <div class="mb-8">
            <h2 class="text-3xl font-bold tracking-tight mb-2">连接</h2>
            <p class="text-muted-foreground">SSH 连接相关设置</p>
        </div>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">身份验证</h3>

            <div class="flex gap-4">
                <div v-for="method in authMethods" :key="method.value"
                    class="flex-1 flex flex-col items-center gap-2 p-4 bg-secondary/20 border-2 border-transparent rounded-xl cursor-pointer transition-all hover:bg-secondary/40"
                    :class="{ 'border-primary bg-primary/5': authMethod === method.value }"
                    @click="authMethod = method.value">
                    <span :class="method.icon" class="text-2xl"
                        :style="{ color: authMethod === method.value ? '#60a5fa' : 'currentColor', opacity: authMethod === method.value ? 1 : 0.7 }"></span>
                    <span class="text-sm font-medium">{{ method.label }}</span>
                </div>
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">超时设置</h3>

            <div class="bg-secondary/20 border border-border rounded-xl overflow-hidden">
                <div
                    class="flex items-center justify-between p-4 border-b border-border hover:bg-foreground/5 transition-colors">
                    <div class="flex flex-col gap-1">
                        <span class="text-sm font-medium">连接超时</span>
                        <span class="text-xs text-muted-foreground">等待服务器响应的最长时间</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="number" v-model="timeout" min="5" max="120"
                            class="w-[70px] bg-background border border-border rounded-md px-3 py-1.5 text-sm text-center focus:ring-2 focus:ring-primary/50 outline-none" />
                        <span class="text-sm text-muted-foreground">秒</span>
                    </div>
                </div>

                <div class="flex items-center justify-between p-4 hover:bg-foreground/5 transition-colors">
                    <div class="flex flex-col gap-1">
                        <span class="text-sm font-medium">心跳间隔</span>
                        <span class="text-xs text-muted-foreground">保持连接活跃的心跳发送间隔</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <input type="number" v-model="keepAlive" min="0" max="300"
                            class="w-[70px] bg-background border border-border rounded-md px-3 py-1.5 text-sm text-center focus:ring-2 focus:ring-primary/50 outline-none" />
                        <span class="text-sm text-muted-foreground">秒</span>
                    </div>
                </div>
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">高级选项</h3>

            <div class="bg-secondary/20 border border-border rounded-xl overflow-hidden">
                <div
                    class="flex items-center justify-between p-4 border-b border-border hover:bg-foreground/5 transition-colors">
                    <div class="flex flex-col gap-1">
                        <span class="text-sm font-medium">数据压缩</span>
                        <span class="text-xs text-muted-foreground">压缩传输数据以节省带宽</span>
                    </div>
                    <Switch v-model:checked="compression" />
                </div>

                <div class="flex items-center justify-between p-4 hover:bg-foreground/5 transition-colors">
                    <span class="text-sm font-medium">SSH 协议版本</span>
                    <div class="flex gap-2">
                        <label
                            class="flex items-center gap-2 px-3 py-1.5 bg-secondary/10 border border-transparent rounded-lg cursor-pointer transition-all hover:bg-secondary/20"
                            :class="{ 'border-primary bg-primary/10': sshVersion === '2' }">
                            <input type="radio" v-model="sshVersion" value="2" class="hidden" />
                            <span class="text-sm">SSH 2</span>
                        </label>
                        <label
                            class="flex items-center gap-2 px-3 py-1.5 bg-secondary/10 border border-transparent rounded-lg cursor-pointer transition-all hover:bg-secondary/20"
                            :class="{ 'border-primary bg-primary/10': sshVersion === '1' }">
                            <input type="radio" v-model="sshVersion" value="1" class="hidden" />
                            <span class="text-sm">SSH 1</span>
                        </label>
                    </div>
                </div>
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">密钥管理</h3>

            <div class="flex flex-col gap-2">
                <div
                    class="flex items-center gap-3 p-3 bg-secondary/20 rounded-xl border border-transparent hover:border-border">
                    <span class="icon-[mdi--key-variant] text-xl text-amber-500"></span>
                    <div class="flex-1 flex flex-col">
                        <span class="text-sm font-medium">id_rsa</span>
                        <span class="text-xs text-muted-foreground font-mono">~/.ssh/id_rsa</span>
                    </div>
                    <span class="px-2.5 py-1 text-xs font-medium bg-green-500/15 text-green-500 rounded-full">已添加</span>
                </div>

                <button
                    class="flex items-center justify-center gap-2 p-3 bg-secondary/20 border border-dashed border-border rounded-xl text-muted-foreground hover:bg-secondary/40 hover:border-border/80 hover:text-foreground transition-all">
                    <span class="icon-[mdi--plus]"></span>
                    <span class="text-sm">添加密钥</span>
                </button>
            </div>
        </section>
    </div>
</template>
