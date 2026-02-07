<script setup lang="ts">
// 壁纸背景组件 - 动态渐变和光效
</script>

<template>
    <div class="wallpaper">
        <span class="glow glow-a"></span>
        <span class="glow glow-b"></span>
        <span class="glow glow-c"></span>
        <span class="wave wave-a"></span>
        <span class="wave wave-b"></span>
    </div>
</template>

<style scoped>
.wallpaper {
    position: absolute;
    inset: 0;
    background: radial-gradient(circle at 20% 20%, rgba(125, 211, 252, 0.14), transparent 45%),
        radial-gradient(circle at 70% 30%, rgba(251, 191, 36, 0.12), transparent 50%),
        linear-gradient(140deg, var(--wallpaper-a), var(--wallpaper-b) 45%, var(--wallpaper-c) 90%);
    z-index: 0;
}

.glow {
    position: absolute;
    width: 380px;
    height: 380px;
    border-radius: 50%;
    filter: blur(60px);
    opacity: 0.7;
}

.glow-a {
    top: -60px;
    left: 10%;
    background: radial-gradient(circle, rgba(94, 234, 212, 0.6), transparent 70%);
}

.glow-b {
    right: 12%;
    top: 22%;
    background: radial-gradient(circle, rgba(125, 211, 252, 0.5), transparent 70%);
}

.glow-c {
    bottom: -120px;
    right: 18%;
    background: radial-gradient(circle, rgba(251, 191, 36, 0.35), transparent 70%);
}

.wave {
    position: absolute;
    width: 120%;
    height: 240px;
    left: -10%;
    border-radius: 999px;
    opacity: 0.22;
    filter: blur(20px);
}

.wave-a {
    bottom: 32%;
    background: linear-gradient(90deg, rgba(125, 211, 252, 0.6), rgba(94, 234, 212, 0.2));
}

.wave-b {
    bottom: 18%;
    background: linear-gradient(90deg, rgba(59, 130, 246, 0.2), rgba(251, 191, 36, 0.5));
}
</style>
