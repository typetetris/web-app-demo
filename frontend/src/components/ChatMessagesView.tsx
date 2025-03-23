import { Flex, View, Switch } from "@adobe/react-spectrum";
import { ChatMessageCard } from "./ChatMessageCard";
import { ChatMessage } from "../util/chatClient";
import { Identity } from "../models/Identity";
import { DOMRefValue, StyleProps } from "@react-types/shared";
import {
  useCallback,
  useDeferredValue,
  useEffect,
  useRef,
  useState,
} from "react";
import { Chat } from "../models/Chat";

export interface ChatMessageViewProps extends StyleProps {
  messages: ChatMessage[];
  identity: Identity;
  chat: Chat;
  ref?: React.RefObject<DOMRefValue<HTMLElement> | null>;
}

function scroll(
  ref: React.RefObject<DOMRefValue<HTMLElement> | null>,
  direction?: "down" | "up",
  behavior?: ScrollBehavior,
) {
  const node = ref.current?.UNSAFE_getDOMNode();
  node?.scroll({ top: direction == "down" ? node.scrollHeight : 0, behavior });
}

function scrollDownSmooth(
  ref: React.RefObject<DOMRefValue<HTMLElement> | null>,
) {
  scroll(ref, "down", "smooth");
}

function scrollDownInstant(
  ref: React.RefObject<DOMRefValue<HTMLElement> | null>,
) {
  scroll(ref, "down", "instant");
}

export function ChatMessagesView({
  messages,
  identity,
  chat,
  ref,
  ...styleProps
}: ChatMessageViewProps) {
  const localRef = useRef<DOMRefValue<HTMLElement>>(null);
  const refToUse = ref != undefined ? ref : localRef;
  const previousLastMessageEventId = useDeferredValue<string | undefined>(
    messages.at(-1)?.event_id,
  );
  const previousChat = useDeferredValue<Chat>(chat);
  const [stayAtMostRecentMessage, setStayAtMostRecentMessage] = useState(true);
  const [initialLoad, setInitialLoad] = useState(true);

  // Clear initial load flag
  useEffect(() => {
    setInitialLoad(false);
  }, []);

  // Scroll to the bottom on own messages or if
  // the chat changes
  useEffect(() => {
    const lastMessage = messages.at(-1);
    const chatChanged = chat.id != previousChat.id;
    const lastMessageChanged =
      lastMessage?.event_id != previousLastMessageEventId;
    const lastMessageIsOwnMessage = lastMessage?.user_id == identity.id;

    if (chatChanged || initialLoad) {
      scrollDownInstant(refToUse);
    } else if (
      lastMessageChanged &&
      (lastMessageIsOwnMessage || stayAtMostRecentMessage)
    ) {
      scrollDownSmooth(refToUse);
      if (lastMessageIsOwnMessage) {
        setStayAtMostRecentMessage(true);
      }
    }
  }, [
    previousLastMessageEventId,
    messages,
    identity,
    refToUse,
    previousChat,
    chat,
    stayAtMostRecentMessage,
    initialLoad,
  ]);

  // Register keydown handler
  const scrollDownHandler = useCallback(
    (ev: KeyboardEvent) => {
      if (ev.getModifierState("Control")) {
        if (ev.key == "ArrowDown") {
          scrollDownSmooth(refToUse);
        } else if (ev.key == "ArrowUp") {
          scroll(refToUse, "up", "smooth");
        }
      }
    },
    [refToUse],
  );
  useEffect(() => {
    window.addEventListener("keydown", scrollDownHandler);
    return () => {
      window.removeEventListener("keydown", scrollDownHandler);
    };
  }, [scrollDownHandler]);

  // Try to deactivate scrollToBottomOnNewMessage if the user
  // scrolls up.
  const scrollTop = useRef<number | null>(null);
  const scrollEventHandler = useCallback(() => {
    const node = refToUse.current?.UNSAFE_getDOMNode();
    if (node == null) {
      scrollTop.current = null;
      return;
    }
    if (scrollTop.current != null && node.scrollTop < scrollTop.current) {
      setStayAtMostRecentMessage(false);
    }
    scrollTop.current = node.scrollTop;
  }, [refToUse, scrollTop]);
  useEffect(() => {
    const node = refToUse.current?.UNSAFE_getDOMNode();
    if (node == null) {
      return () => {};
    }
    node.addEventListener("scroll", scrollEventHandler);
    return () => {
      node.removeEventListener("scroll", scrollEventHandler);
    };
  }, [scrollEventHandler, refToUse]);

  return (
    <>
      <View overflow="scroll" {...styleProps} ref={refToUse}>
        <Flex direction="column" gap="size-300">
          {messages.map((message) => (
            <ChatMessageCard
              key={message.event_id}
              message={message}
              alignSelf={
                identity.id == message.user_id ? "flex-end" : "flex-start"
              }
              printDisplayName={identity.id != message.user_id}
            ></ChatMessageCard>
          ))}
        </Flex>
      </View>
      <Flex direction="row">
        <Switch
          isSelected={stayAtMostRecentMessage}
          onChange={(isSelected) => {
            setStayAtMostRecentMessage(isSelected);
            if (isSelected) {
              scrollDownSmooth(refToUse);
            }
          }}
        >
          Bei aktuellster Nachricht bleiben
        </Switch>
      </Flex>
    </>
  );
}
