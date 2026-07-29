import { ref, onUnmounted } from 'vue';

export function useSse(onEvent: (event: any) => void) {
  const isConnected = ref(false);
  let eventSource: EventSource | null = null;

  const connect = () => {
    eventSource = new EventSource('/api/events');

    eventSource.onopen = () => {
      isConnected.value = true;
    };

    eventSource.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data);
        onEvent(data);
      } catch (err) {
        console.error('Failed to parse SSE data', err);
      }
    };

    eventSource.onerror = () => {
      isConnected.value = false;
    };
  };

  const disconnect = () => {
    if (eventSource) {
      eventSource.close();
      eventSource = null;
      isConnected.value = false;
    }
  };

  onUnmounted(() => {
    disconnect();
  });

  return {
    isConnected,
    connect,
    disconnect,
  };
}
