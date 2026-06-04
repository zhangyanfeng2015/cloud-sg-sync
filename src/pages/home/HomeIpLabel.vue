<script setup lang="ts">
import { ref } from "vue";
import { Refresh } from "@element-plus/icons-vue";

const loading = ref(false);

const emit = defineEmits<{
  refresh: [];
}>();

function onClick() {
  if (loading.value) return;
  emit("refresh");
}

function setLoading(value: boolean) {
  loading.value = value;
}

defineExpose({ setLoading });
</script>

<template>
  <div class="ip-hero-label-group">
    <span class="ip-hero-label">本机当前公网 IP</span>
    <el-tooltip content="重新探测公网 IP" placement="top" :show-after="300">
      <button
        type="button"
        class="ip-refresh-btn"
        aria-label="刷新公网 IP"
        :disabled="loading"
        @click="onClick"
      >
        <el-icon :class="{ 'is-loading': loading }" :size="14">
          <Refresh />
        </el-icon>
      </button>
    </el-tooltip>
  </div>
</template>
