import { defineStore } from 'pinia';

interface DevicePollDebugState {
  visible: boolean;
  autoPopupOnError: boolean;
  unreadErrors: number;
}

export const useDevicePollDebugStore = defineStore('device-poll-debug-store', {
  state: (): DevicePollDebugState => ({
    visible: false,
    autoPopupOnError: true,
    unreadErrors: 0
  }),
  actions: {
    open() {
      this.visible = true;
      this.unreadErrors = 0;
    },
    close() {
      this.visible = false;
    },
    toggle() {
      this.visible = !this.visible;
      if (this.visible) this.unreadErrors = 0;
    },
    notifyError() {
      if (this.visible) return;
      this.unreadErrors += 1;
      if (this.autoPopupOnError) {
        this.visible = true;
        this.unreadErrors = 0;
      }
    },
    setAutoPopup(v: boolean) {
      this.autoPopupOnError = v;
    }
  }
});
