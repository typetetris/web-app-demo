import {
  ActionButton,
  Flex,
  Heading,
  ProgressCircle,
  ToastQueue,
  Tooltip,
  TooltipTrigger,
} from "@adobe/react-spectrum";
import { Identity } from "../models/Identity";
import { ChatMessage } from "../util/chatClient";
import { DOMRef, StyleProps } from "@react-types/shared";
import { Chat } from "../models/Chat";
import { ChatMessagesView } from "./ChatMessagesView";
import { SendChatMessageForm } from "./SendChatMessageForm";
import { useRef } from "react";
import ShareAndroid from "@spectrum-icons/workflow/ShareAndroid";
import { useMutation } from "@tanstack/react-query";

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
  const copyToClipboardAction = useMutation({
    mutationFn: () => navigator.clipboard.writeText(location.href),
    retry: 0,
    onSettled: () => {
      ToastQueue.positive(
        `Url für den Chat ${chat.name} in die Zwischenablage kopiert`,
        { timeout: 5000 },
      );
    },
  });
  return (
    <Flex direction="column" justifyContent="stretch" {...styleProps}>
      <Flex
        direction="row"
        justifyContent="space-between"
        alignItems="baseline"
        flex="0 0 auto"
      >
        <Flex direction="row" alignItems="center" gap="size-100">
          {copyToClipboardAction.isPending ? (
            <ProgressCircle
              aria-label="Kopiervorgang läuft..."
              isIndeterminate
            />
          ) : (
            <ShareButton onPress={() => copyToClipboardAction.mutate()} />
          )}
          <Heading level={2}>{chat.name}</Heading>
        </Flex>
        <Heading level={2}>{identity.displayName}</Heading>
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

interface ShareButtonProps {
  onPress: () => void;
}

function ShareButton({ onPress }: ShareButtonProps) {
  return (
    <TooltipTrigger>
      <ActionButton isQuiet onPress={onPress}>
        <ShareAndroid />
      </ActionButton>
      <Tooltip>Chat Url in Zwischenablage kopieren</Tooltip>
    </TooltipTrigger>
  );
}
