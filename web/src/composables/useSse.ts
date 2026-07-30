import { ref, onUnmounted } from 'vue';

export function useSse(onEvent: (event: any) => void) {
  const isConnected = ref(false);
  let eventSource: EventSource | null = null;

  const connect = () => {
    if (eventSource) {
      eventSource.close();
    }

    eventSource = new EventSource('/api/events');

    eventSource.onopen = () => {
      isConnected.value = true;
    };

    eventSource.onmessage = (msg) => {
      // Whenever any message arrives (including {"type":"connected"}), connection is active
      isConnected.value = true;
      try {
        const data = JSON.parse(msg.data);
        if (data && data.type !== 'connected') {
          onEvent(data);
        }
      } catch (err) {
        console.error('Failed to parse SSE data', err);
      }
    };

    eventSource.onerror = () => {
      // Only set isConnected = false if the socket stream was actually CLOSED
      if (eventSource && eventSource.readyState === EventSource.CLOSED) {
        isConnected.value = false;
      }
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
