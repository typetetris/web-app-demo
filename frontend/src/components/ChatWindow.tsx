import { Flex, Heading, View } from "@adobe/react-spectrum";
import { Identity } from "../models/Identity";
import { ChatMessage } from "../util/chatClient";
import { DOMRef, StyleProps } from "@react-types/shared";
import { Chat } from "../models/Chat";
import { ChatMessagesView } from "./ChatMessagesView";
import { SendChatMessageForm } from "./SendChatMessageForm";
import { useRef } from "react";

export interface ChatWindowProps extends StyleProps {
  messages: ChatMessage[];
  identity: Identity;
  chat: Chat;
  onSend: (msg: string) => void;
}
export function ChatWindow({
  messages,
  identity,
  chat,
  onSend,
  ...styleProps
}: ChatWindowProps) {
  const chatMessagesViewRef: DOMRef = useRef(null);
  return (
    <Flex direction="column" justifyContent="stretch" {...styleProps}>
      <Flex
        direction="row"
        justifyContent="space-between"
        alignItems="baseline"
        flex="0 0 auto"
      >
        <Heading level={2}>{chat.name}</Heading>
        <View>Sie chatten als: {identity.displayName}</View>
      </Flex>
      <ChatMessagesView
        messages={messages}
        identity={identity}
        chat={chat}
        flex="1 1 auto"
        ref={chatMessagesViewRef}
      />
      <SendChatMessageForm
        onSubmit={onSend}
        onScrollToBottom={() => {}}
        flex="0 0 auto"
        marginX="size-300"
      />
    </Flex>
  );
}
