import { ensureNever } from "./util";

// As I didn't know about react-use-websocket and flushSync, implementing
// this with useEffect, useState and so on was a nightmare.
//
// Therefore I went for useSyncExternalStore.
//
// This also has the benefit, that ChatClient is agnostic about the framework
// and could be turned into its own library for different use cases.

const SEND_ON_IS_PENDING_ERROR_MESSAGE = "cannot send isPending";
export const pendingChatClientState = {
  isPending: true,
  isClosed: false,
  closeCode: null,
  closeReason: null,
  isError: false,
  error: null,
  messages: [],
  send: (_: string) => {
    throw new Error(SEND_ON_IS_PENDING_ERROR_MESSAGE);
  },
};
export class ChatClient {
  private webSocketPending = true;
  private historyPending = true;

  private isError = false;
  private error: unknown = null;

  private isClosed = false;
  private closeReason: string | null = null;
  private closeCode: number | null = null;

  private displayName: string;

  private messages: ChatMessage[] = [];

  private webSocket: WebSocket | null = null;

  private snapshot: ChatClientSnapshot = pendingChatClientState;

  private onSnapshotChangeHandler: () => void = () => {};
  private dispatchSnapshotChange = () => {
    this.snapshot = {
      isPending:
        !this.isError && (this.webSocketPending || this.historyPending),
      isError: this.isError,
      error: this.error,
      isClosed: this.isClosed,
      closeReason: this.closeReason,
      closeCode: this.closeCode,
      messages: [...this.messages],
      send: this.send,
    };
    this.onSnapshotChangeHandler();
  };
  public onSnapshotChange(handler: () => void) {
    this.onSnapshotChangeHandler = handler;
  }

  constructor(chatId: string, userId: string, displayName: string) {
    this.displayName = displayName;
    try {
      const ws = new WebSocket(`${ENDPOINT}/chat/${chatId}/${userId}`);
      ws.addEventListener("close", this.onClose);
      ws.addEventListener("message", (event) => {
        this.onMessage(event);
      });
      ws.addEventListener("open", (event) => {
        this.onOpen(event);
      });
      ws.addEventListener("error", (event) => {
        this.onError(event);
      });
      this.webSocket = ws;
    } catch (e) {
      this.isError = true;
      this.error = e;
      // So the user has a chance to attach a onSnapshotChange callback
      // before the snapshot change is dispatched.
      new Promise<void>((resolve) => {
        this.dispatchSnapshotChange();
        resolve();
      });
    }
    fetch(`${ENDPOINT}/history/${chatId}`)
      .then(async (response) => {
        if (response.status != 404 && !response.ok) {
          let errorBody = null;
          try {
            errorBody = await response.text();
          } catch (e) {
            errorBody = `error receiving error body: ${e}`;
          }
          throw new Error(
            `error fetching history: ${response.status} ${response.statusText} ${errorBody}`,
          );
        }
        if (response.ok) {
          const body = await response.text();
          const wireMessages = (JSON.parse(body) as RawChatMessage[]).map(
            convertChatMessageFromWire,
          );
          this.onHistory(wireMessages);
        }
      })
      .catch((reason) => {
        this.isError = true;
        this.error = reason;
        this.historyPending = false;
        this.dispatchSnapshotChange();
      });
  }

  private onClose = (event: CloseEvent) => {
    this.webSocket?.removeEventListener("error", this.onError);
    this.webSocket?.removeEventListener("open", this.onOpen);
    this.webSocket?.removeEventListener("message", this.onMessage);
    this.webSocket?.removeEventListener("close", this.onClose);

    this.isClosed = true;
    this.closeCode = event.code;
    this.closeReason = event.reason;
    this.dispatchSnapshotChange();
  };

  private onError = (event: Event) => {
    this.isError = true;
    this.error = event;
    this.dispatchSnapshotChange();
  };
  private onMessage = (event: MessageEvent) => {
    try {
      if (typeof event.data == "string") {
        const message = JSON.parse(event.data) as Outgoing;
        const messageType = message.type;
        switch (messageType) {
          case "ChatMessage":
            this.messages.push(convertChatMessageFromWire(message.msg));
            break;
          case "Error":
            throw new Error(`error from server received: ${message.msg}`);
          default:
            ensureNever(messageType);
            // Should be impossible, but because we casted the parsed
            // json above, we don't really know, what we got.
            throw new Error(`unknown message type received: ${message}`);
        }
      } else {
        throw new Error(`unexpected websocket message received: ${event.data}`);
      }
    } catch (e) {
      this.isError = true;
      this.error = e;
    }
    this.dispatchSnapshotChange();
  };
  private onOpen = (_: Event) => {
    this.webSocketPending = false;
    this.dispatchSnapshotChange();
  };
  private onHistory = (history: ChatMessage[]) => {
    const combinedMessages = new Map<string, ChatMessage>();
    for (const message of history) {
      combinedMessages.set(message.event_id, message);
    }
    for (const message of this.messages) {
      combinedMessages.set(message.event_id, message);
    }
    this.messages = [...combinedMessages.values()].sort(
      (a: ChatMessage, b: ChatMessage) =>
        b.timestamp.getDate() - a.timestamp.getDate(),
    );
    this.historyPending = false;
    this.dispatchSnapshotChange();
  };
  public getSnapshot(): ChatClientSnapshot {
    return this.snapshot;
  }
  public send = (message: string) => {
    const ws = this.webSocket;
    if (this.isError) {
      throw new Error("cannot send isError");
    } else if (this.webSocketPending) {
      throw new Error(SEND_ON_IS_PENDING_ERROR_MESSAGE);
    } else if (this.isClosed) {
      throw new Error("cannot send isClosed");
    } else if (ws == null) {
      throw new Error("cannot send websocket is null");
    }
    ws.send(
      JSON.stringify({
        display_name: this.displayName,
        message,
      } satisfies IncomingChatMessage),
    );
  };

  public close() {
    const ws = this.webSocket;
    if (ws == null) {
      return;
    }
    if (
      ws.readyState != WebSocket.CLOSED &&
      ws.readyState != WebSocket.CLOSING
    ) {
      ws.close();
    }
  }
}

export interface ChatClientSnapshot {
  isPending: boolean;
  isError: boolean;
  error: unknown;
  isClosed: boolean;
  closeReason: string | null;
  closeCode: number | null;
  messages: ChatMessage[];
  send: (msg: string) => void;
}

const ENDPOINT = "http://localhost:8080";

interface RawChatMessage {
  event_id: string;
  timestamp: string;
  chat_id: string;
  user_id: string;
  display_name: string;
  message: string;
}

export interface ChatMessage {
  event_id: string;
  timestamp: Date;
  chat_id: string;
  user_id: string;
  display_name: string;
  message: string;
}

function convertChatMessageFromWire(wireMessage: RawChatMessage): ChatMessage {
  return {
    ...wireMessage,
    timestamp: new Date(wireMessage.timestamp),
  };
}

interface OutgoingChatMessage {
  type: "ChatMessage";
  msg: RawChatMessage;
}

interface OutgoingError {
  type: "Error";
  msg: string;
}

type Outgoing = OutgoingChatMessage | OutgoingError;

export interface IncomingChatMessage {
  display_name: string;
  message: string;
}
