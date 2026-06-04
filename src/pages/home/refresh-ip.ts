import { createApp, h, ref, type App } from "vue";
import HomeIpLabel from "./HomeIpLabel.vue";

export interface RefreshIpButtonApi {
  setLoading: (value: boolean) => void;
}

interface HomeIpLabelExposed {
  setLoading: (value: boolean) => void;
}

let app: App | null = null;
let api: RefreshIpButtonApi | null = null;

export function mountRefreshIpButton(
  onRefresh: () => void | Promise<void>,
): RefreshIpButtonApi | null {
  const host = document.getElementById("homeIpLabelMount");
  if (!host || app) return api;

  const labelRef = ref<HomeIpLabelExposed | null>(null);

  app = createApp({
    render: () =>
      h(HomeIpLabel, {
        ref: (el: unknown) => {
          labelRef.value = el as HomeIpLabelExposed;
        },
        onRefresh: () => void onRefresh(),
      }),
  });
  app.mount(host);

  api = {
    setLoading: (value: boolean) => labelRef.value?.setLoading(value),
  };
  return api;
}

export function getRefreshIpButtonApi(): RefreshIpButtonApi | null {
  return api;
}
