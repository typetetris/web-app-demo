import { Content, Flex, Text, View } from "@adobe/react-spectrum";
import { ChatMessage } from "../util/chatClient";
import { StyleProps } from "@react-types/shared";
import { CSSProperties } from "react";
import Markdown from "react-markdown";

const dateTimeFormatter = new Intl.DateTimeFormat("de", {
  dateStyle: "full",
  timeStyle: "medium",
});

const textStyle: CSSProperties = {
  fontSize: "12px",
};

export interface ChatMessageProps extends StyleProps {
  message: ChatMessage;
  printDisplayName?: boolean;
}
export function ChatMessageCard({
  message,
  printDisplayName,
  ...styleProps
}: ChatMessageProps) {
  return (
    <View
      borderWidth="thin"
      borderColor="dark"
      borderRadius="medium"
      padding="size-150"
      backgroundColor="gray-50"
      maxWidth="min(75%, size-6000)"
      {...styleProps}
    >
      <Flex direction="column">
        {(printDisplayName ?? true) ? (
          <Text UNSAFE_style={textStyle} alignSelf="flex-start">
            {message.display_name}
          </Text>
        ) : null}
        <Content>
          <Markdown>{message.message}</Markdown>
        </Content>
        <Text UNSAFE_style={textStyle} alignSelf="flex-end">
          {dateTimeFormatter.format(message.timestamp)}
        </Text>
      </Flex>
    </View>
  );
}
