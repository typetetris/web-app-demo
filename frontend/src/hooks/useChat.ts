import { useMemo, useSyncExternalStore } from "react";
import {
  ChatClient,
  ChatClientSnapshot,
  pendingChatClientState,
} from "../util/chatClient";

interface ChatClientBox {
  current: ChatClient | null;
}

interface Store {
  subscribe: (onStoreChange: () => void) => () => void;
  getSnapshot: () => ChatClientSnapshot;
}

const emptyStore: Store = {
  subscribe: () => {
    return () => {};
  },
  getSnapshot: () => {
    return pendingChatClientState;
  },
};

function createStore(
  chatId: string,
  userId: string,
  displayName: string,
): Store {
  const chatClientBox: ChatClientBox = { current: null };
  return {
    subscribe: (onStoreChange: () => void) => {
      chatClientBox.current = new ChatClient(chatId, userId, displayName);
      chatClientBox.current.onSnapshotChange(onStoreChange);
      return () => {
        chatClientBox.current?.close();
      };
    },
    getSnapshot: () => {
      return chatClientBox.current?.getSnapshot() ?? pendingChatClientState;
    },
  };
}

export function useChat(
  chatId: string | null,
  userId: string | null,
  displayName: string | null,
) {
  const { subscribe, getSnapshot } = useMemo<Store>(() => {
    if (chatId == null || userId == null || displayName == null) {
      return emptyStore;
    } else {
      return createStore(chatId, userId, displayName);
    }
  }, [chatId, userId, displayName]);
  const snapshot = useSyncExternalStore(subscribe, getSnapshot);
  return snapshot;
}
